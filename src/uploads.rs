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

const MIB: usize = 1024 * 1024;
pub const MAX_IMAGE_BYTES: usize = 10 * MIB;
pub const MAX_VIDEO_BYTES: usize = 50 * MIB;
pub const MAX_MULTIPART_BYTES: usize = MAX_VIDEO_BYTES + MIB;
const PRESIGNED_URL_TTL: Duration = Duration::from_secs(60 * 60);
const RANDOM_KEY_BYTES: usize = 18;

const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/avif",
];
const ALLOWED_VIDEO_TYPES: &[&str] = &["video/mp4", "video/webm", "video/quicktime"];

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
    MissingContentType,
    UnsupportedContentType(String),
    EmptyFile,
    TooLarge {
        media_kind: &'static str,
        limit_bytes: usize,
    },
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

        let content_type = field
            .content_type()
            .map(|value| value.to_string())
            .ok_or(UploadError::MissingContentType)?;
        let validation = validate_media_content_type(&content_type)?;
        let body = read_limited_field(field, validation.max_bytes, validation.media_kind).await?;
        if body.is_empty() {
            return Err(UploadError::EmptyFile);
        }
        let size_bytes = i64::try_from(body.len()).map_err(|_| UploadError::BodyTooLarge)?;
        let object_key = build_object_key(validation.extension)?;

        let stored = storage
            .put_object(&object_key, body, Some(&validation.content_type))
            .await
            .map_err(UploadError::Storage)?;
        let media = media::create_media(
            &pool,
            CreateMediaInput {
                object_key: stored.relative_key,
                media_type: validation.media_type,
                content_type: validation.content_type,
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
            Self::MissingFile
            | Self::MissingContentType
            | Self::UnsupportedContentType(_)
            | Self::EmptyFile => StatusCode::BAD_REQUEST,
            Self::TooLarge { .. } | Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Multipart(_)
            | Self::Randomness(_)
            | Self::Storage(_)
            | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::MissingFile => "A file field is required.".to_owned(),
            Self::MissingContentType => {
                "Uploaded files must include a content type.".to_owned()
            }
            Self::UnsupportedContentType(_) => format!(
                "Unsupported file type. Allowed image types: {}. Allowed video types: {}.",
                ALLOWED_IMAGE_TYPES.join(", "),
                ALLOWED_VIDEO_TYPES.join(", ")
            ),
            Self::EmptyFile => "The uploaded file is empty.".to_owned(),
            Self::TooLarge {
                media_kind,
                limit_bytes,
            } => format!(
                "{} uploads must be {} MiB or smaller.",
                media_kind,
                limit_bytes / MIB
            ),
            Self::BodyTooLarge => "The uploaded file is too large.".to_owned(),
            Self::Multipart(_)
            | Self::Randomness(_)
            | Self::Storage(_)
            | Self::Database(_) => "Upload failed.".to_owned(),
        }
    }
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile => write!(f, "multipart request did not contain a file field"),
            Self::Multipart(error) => write!(f, "failed to read multipart upload: {error}"),
            Self::MissingContentType => write!(f, "uploaded file did not include a content type"),
            Self::UnsupportedContentType(content_type) => {
                write!(f, "unsupported media content type: {content_type}")
            }
            Self::EmptyFile => write!(f, "uploaded file is empty"),
            Self::TooLarge {
                media_kind,
                limit_bytes,
            } => {
                write!(f, "{media_kind} upload exceeds {limit_bytes} bytes")
            }
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
            | Self::MissingContentType
            | Self::UnsupportedContentType(_)
            | Self::EmptyFile
            | Self::TooLarge { .. }
            | Self::BodyTooLarge => None,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn read_limited_field(
    mut field: axum::extract::multipart::Field<'_>,
    max_bytes: usize,
    media_kind: &'static str,
) -> Result<Vec<u8>, UploadError> {
    let mut body = Vec::new();

    while let Some(chunk) = field.chunk().await.map_err(UploadError::Multipart)? {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(UploadError::BodyTooLarge)?;
        if next_len > max_bytes {
            return Err(UploadError::TooLarge {
                media_kind,
                limit_bytes: max_bytes,
            });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

struct ValidatedMedia {
    media_type: MediaType,
    content_type: String,
    max_bytes: usize,
    media_kind: &'static str,
    extension: &'static str,
}

fn validate_media_content_type(content_type: &str) -> Result<ValidatedMedia, UploadError> {
    let normalized = normalize_content_type(content_type);
    let (media_type, max_bytes, media_kind, extension) = match normalized.as_str() {
        "image/jpeg" => (MediaType::Image, MAX_IMAGE_BYTES, "Image", "jpg"),
        "image/png" => (MediaType::Image, MAX_IMAGE_BYTES, "Image", "png"),
        "image/gif" => (MediaType::Image, MAX_IMAGE_BYTES, "Image", "gif"),
        "image/webp" => (MediaType::Image, MAX_IMAGE_BYTES, "Image", "webp"),
        "image/avif" => (MediaType::Image, MAX_IMAGE_BYTES, "Image", "avif"),
        "video/mp4" => (MediaType::Video, MAX_VIDEO_BYTES, "Video", "mp4"),
        "video/webm" => (MediaType::Video, MAX_VIDEO_BYTES, "Video", "webm"),
        "video/quicktime" => (MediaType::Video, MAX_VIDEO_BYTES, "Video", "mov"),
        _ => return Err(UploadError::UnsupportedContentType(normalized)),
    };

    Ok(ValidatedMedia {
        media_type,
        content_type: normalized,
        max_bytes,
        media_kind,
        extension,
    })
}

fn normalize_content_type(content_type: &str) -> String {
    match content_type.split(';').next() {
        Some(value) => value.trim().to_ascii_lowercase(),
        None => String::new(),
    }
}

fn build_object_key(extension: &str) -> Result<String, UploadError> {
    let mut random = [0_u8; RANDOM_KEY_BYTES];
    getrandom(&mut random).map_err(UploadError::Randomness)?;

    Ok(format!(
        "media/{}-{}.{}",
        OffsetDateTime::now_utc().unix_timestamp(),
        URL_SAFE_NO_PAD.encode(random),
        extension
    ))
}
