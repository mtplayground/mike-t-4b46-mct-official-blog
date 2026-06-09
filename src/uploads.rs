use std::{error::Error, fmt, time::Duration};

use axum::{
    Extension, Json,
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use getrandom::getrandom;
use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    repositories::media::{self, CreateMediaInput, MediaType},
    storage::{ObjectStorage, StorageError},
};

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_MULTIPART_BYTES: usize = MAX_UPLOAD_BYTES + 1024 * 1024;
const PRESIGNED_URL_TTL: Duration = Duration::from_secs(60 * 60);
const RANDOM_KEY_BYTES: usize = 18;

#[derive(Serialize)]
pub struct UploadResponse {
    pub id: i64,
    pub object_key: String,
    pub object_url: String,
    pub media_type: String,
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Debug)]
pub enum UploadError {
    MissingFile,
    Multipart(axum::extract::multipart::MultipartError),
    UnsupportedContentType(String),
    EmptyFile,
    TooLarge,
    BodyTooLarge,
    Randomness(getrandom::Error),
    Storage(StorageError),
    Database(sqlx::Error),
}

pub async fn upload_media(
    Extension(pool): Extension<PgPool>,
    Extension(storage): Extension<ObjectStorage>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, UploadError> {
    while let Some(field) = multipart.next_field().await.map_err(UploadError::Multipart)? {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.filename().map(ToOwned::to_owned);
        let content_type = field
            .content_type()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_owned());
        let media_type = media_type_from_content_type(&content_type)?;
        let body = read_limited_field(field).await?;
        if body.is_empty() {
            return Err(UploadError::EmptyFile);
        }
        let size_bytes = i64::try_from(body.len()).map_err(|_| UploadError::BodyTooLarge)?;
        let object_key = build_object_key(filename.as_deref(), &content_type)?;

        let stored = storage
            .put_object(&object_key, body, Some(&content_type))
            .await
            .map_err(UploadError::Storage)?;
        let media = media::create_media(
            &pool,
            CreateMediaInput {
                object_key: stored.relative_key,
                media_type,
                content_type,
                size_bytes,
            },
        )
        .await
        .map_err(UploadError::Database)?;
        let object_url = storage
            .presigned_get_url(&media.object_key, PRESIGNED_URL_TTL)
            .await
            .map_err(UploadError::Storage)?;

        return Ok(Json(UploadResponse {
            id: media.id,
            object_key: media.object_key,
            object_url,
            media_type: media.media_type,
            content_type: media.content_type,
            size_bytes: media.size_bytes,
        }));
    }

    Err(UploadError::MissingFile)
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        eprintln!("upload failed: {self}");

        (status, Json(ErrorResponse { error: self.message() })).into_response()
    }
}

impl UploadError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingFile | Self::UnsupportedContentType(_) | Self::EmptyFile => {
                StatusCode::BAD_REQUEST
            }
            Self::TooLarge | Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Multipart(_)
            | Self::Randomness(_)
            | Self::Storage(_)
            | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::MissingFile => "A file field is required.",
            Self::UnsupportedContentType(_) => "Only image and video uploads are supported.",
            Self::EmptyFile => "The uploaded file is empty.",
            Self::TooLarge | Self::BodyTooLarge => "The uploaded file is too large.",
            Self::Multipart(_)
            | Self::Randomness(_)
            | Self::Storage(_)
            | Self::Database(_) => "Upload failed.",
        }
    }
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile => write!(f, "multipart request did not contain a file field"),
            Self::Multipart(error) => write!(f, "failed to read multipart upload: {error}"),
            Self::UnsupportedContentType(content_type) => {
                write!(f, "unsupported media content type: {content_type}")
            }
            Self::EmptyFile => write!(f, "uploaded file is empty"),
            Self::TooLarge => write!(f, "uploaded file exceeds {MAX_UPLOAD_BYTES} bytes"),
            Self::BodyTooLarge => write!(f, "uploaded file length does not fit in i64"),
            Self::Randomness(error) => write!(f, "failed to generate object key: {error}"),
            Self::Storage(error) => write!(f, "{error}"),
            Self::Database(error) => write!(f, "failed to record media metadata: {error}"),
        }
    }
}

impl Error for UploadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Multipart(error) => Some(error),
            Self::Randomness(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::Database(error) => Some(error),
            Self::MissingFile
            | Self::UnsupportedContentType(_)
            | Self::EmptyFile
            | Self::TooLarge
            | Self::BodyTooLarge => None,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

async fn read_limited_field(
    mut field: axum::extract::multipart::Field<'_>,
) -> Result<Vec<u8>, UploadError> {
    let mut body = Vec::new();

    while let Some(chunk) = field.chunk().await.map_err(UploadError::Multipart)? {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(UploadError::TooLarge)?;
        if next_len > MAX_UPLOAD_BYTES {
            return Err(UploadError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

fn media_type_from_content_type(content_type: &str) -> Result<MediaType, UploadError> {
    if content_type.starts_with("image/") {
        Ok(MediaType::Image)
    } else if content_type.starts_with("video/") {
        Ok(MediaType::Video)
    } else {
        Err(UploadError::UnsupportedContentType(content_type.to_owned()))
    }
}

fn build_object_key(filename: Option<&str>, content_type: &str) -> Result<String, UploadError> {
    let mut random = [0_u8; RANDOM_KEY_BYTES];
    getrandom(&mut random).map_err(UploadError::Randomness)?;
    let extension = file_extension(filename).or_else(|| content_type_extension(content_type));

    Ok(format!(
        "media/{}-{}{}",
        OffsetDateTime::now_utc().unix_timestamp(),
        URL_SAFE_NO_PAD.encode(random),
        extension.unwrap_or_default()
    ))
}

fn file_extension(filename: Option<&str>) -> Option<String> {
    let extension = filename?.rsplit_once('.')?.1.to_ascii_lowercase();
    if extension.is_empty()
        || extension.len() > 12
        || !extension.chars().all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }

    Some(format!(".{extension}"))
}

fn content_type_extension(content_type: &str) -> Option<String> {
    let extension = match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        _ => return None,
    };

    Some(format!(".{extension}"))
}
