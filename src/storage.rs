use std::{
    error::Error,
    fmt,
    time::{Duration, SystemTime},
};

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, RequestChecksumCalculation},
    presigning::PresigningConfig,
    primitives::ByteStream,
};
use aws_types::region::Region;

use crate::config::ObjectStorageConfig;

const MAX_PRESIGN_DURATION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

type BoxError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
pub struct ObjectStorage {
    client: Client,
    config: ObjectStorageConfig,
}

pub struct StoredObject {
    pub bucket: String,
    pub relative_key: String,
    pub full_key: String,
    pub content_length: usize,
    pub e_tag: Option<String>,
}

pub struct ObjectBytes {
    pub bucket: String,
    pub relative_key: String,
    pub full_key: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
}

#[derive(Debug)]
pub enum StorageError {
    EmptyKey,
    InvalidPresignDuration,
    BodyTooLarge,
    Put { source: BoxError },
    Get { source: BoxError },
    ReadBody { source: BoxError },
    PresignConfig { source: BoxError },
    Presign { source: BoxError },
}

impl ObjectStorage {
    pub fn new(config: &ObjectStorageConfig) -> Self {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None::<SystemTime>,
            "object-storage-env",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style)
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            config: config.clone(),
        }
    }

    pub async fn put_object(
        &self,
        relative_key: &str,
        body: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<StoredObject, StorageError> {
        let full_key = self.full_key(relative_key)?;
        let content_length = body.len();
        let content_length_i64 =
            i64::try_from(content_length).map_err(|_| StorageError::BodyTooLarge)?;

        let mut request = self
            .client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&full_key)
            .content_length(content_length_i64)
            .body(ByteStream::from(body));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        let output = request
            .send()
            .await
            .map_err(|source| StorageError::Put {
                source: Box::new(source),
            })?;

        Ok(StoredObject {
            bucket: self.config.bucket.clone(),
            relative_key: normalize_relative_key(relative_key)?,
            full_key,
            content_length,
            e_tag: output.e_tag().map(ToOwned::to_owned),
        })
    }

    pub async fn get_object(&self, relative_key: &str) -> Result<ObjectBytes, StorageError> {
        let full_key = self.full_key(relative_key)?;
        let output = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|source| StorageError::Get {
                source: Box::new(source),
            })?;

        let content_type = output.content_type().map(ToOwned::to_owned);
        let content_length = output.content_length();
        let bytes = output
            .body
            .collect()
            .await
            .map_err(|source| StorageError::ReadBody {
                source: Box::new(source),
            })?
            .into_bytes()
            .to_vec();

        Ok(ObjectBytes {
            bucket: self.config.bucket.clone(),
            relative_key: normalize_relative_key(relative_key)?,
            full_key,
            bytes,
            content_type,
            content_length,
        })
    }

    pub async fn presigned_get_url(
        &self,
        relative_key: &str,
        expires_in: Duration,
    ) -> Result<String, StorageError> {
        if expires_in.is_zero() || expires_in > MAX_PRESIGN_DURATION {
            return Err(StorageError::InvalidPresignDuration);
        }

        let full_key = self.full_key(relative_key)?;
        let presigning_config =
            PresigningConfig::expires_in(expires_in).map_err(|source| {
                StorageError::PresignConfig {
                    source: Box::new(source),
                }
            })?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&full_key)
            .presigned(presigning_config)
            .await
            .map_err(|source| StorageError::Presign {
                source: Box::new(source),
            })?;

        Ok(presigned.uri().to_string())
    }

    pub fn full_key(&self, relative_key: &str) -> Result<String, StorageError> {
        let relative_key = normalize_relative_key(relative_key)?;
        Ok(self.config.full_key(&relative_key))
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => write!(f, "object key must not be empty"),
            Self::InvalidPresignDuration => {
                write!(f, "presigned URL duration must be between 1 second and 7 days")
            }
            Self::BodyTooLarge => write!(f, "object body is too large"),
            Self::Put { .. } => write!(f, "failed to put object into Object Storage"),
            Self::Get { .. } => write!(f, "failed to get object from Object Storage"),
            Self::ReadBody { .. } => write!(f, "failed to read object body from Object Storage"),
            Self::PresignConfig { .. } => write!(f, "failed to configure presigned URL"),
            Self::Presign { .. } => write!(f, "failed to create presigned Object Storage URL"),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Put { source }
            | Self::Get { source }
            | Self::ReadBody { source }
            | Self::PresignConfig { source }
            | Self::Presign { source } => Some(source.as_ref()),
            Self::EmptyKey | Self::InvalidPresignDuration | Self::BodyTooLarge => None,
        }
    }
}

fn normalize_relative_key(relative_key: &str) -> Result<String, StorageError> {
    let relative_key = relative_key.trim_start_matches('/');
    if relative_key.is_empty() {
        return Err(StorageError::EmptyKey);
    }

    Ok(relative_key.to_owned())
}
