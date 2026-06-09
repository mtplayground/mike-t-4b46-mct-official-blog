use std::{error::Error, fmt};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;

use crate::config::{AdminConfig, SessionConfig};

type HmacSha256 = Hmac<Sha256>;

pub const ADMIN_SESSION_COOKIE: &str = "mct_admin_session";
pub const SESSION_TTL_SECONDS: i64 = 60 * 60 * 12;

const SESSION_VERSION: &str = "v1";
const SESSION_NONCE_BYTES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminSession {
    pub username: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    InvalidSession,
    ExpiredSession,
    Signing,
    Randomness(getrandom::Error),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "invalid admin credentials"),
            Self::InvalidSession => write!(f, "invalid admin session"),
            Self::ExpiredSession => write!(f, "admin session has expired"),
            Self::Signing => write!(f, "failed to sign admin session"),
            Self::Randomness(_) => write!(f, "failed to generate admin session nonce"),
        }
    }
}

impl Error for AuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Randomness(error) => Some(error),
            Self::InvalidCredentials
            | Self::InvalidSession
            | Self::ExpiredSession
            | Self::Signing => None,
        }
    }
}

pub fn validate_admin_credentials(
    config: &AdminConfig,
    username: &str,
    password: &str,
) -> Result<(), AuthError> {
    if constant_time_eq(username.as_bytes(), config.username.as_bytes())
        && constant_time_eq(password.as_bytes(), config.password.as_bytes())
    {
        Ok(())
    } else {
        Err(AuthError::InvalidCredentials)
    }
}

pub fn authenticate_admin_cookie(
    admin: &AdminConfig,
    session: &SessionConfig,
    username: &str,
    password: &str,
) -> Result<String, AuthError> {
    validate_admin_credentials(admin, username, password)?;
    create_admin_session_cookie(admin, session)
}

pub fn create_admin_session_token(
    admin: &AdminConfig,
    session: &SessionConfig,
) -> Result<String, AuthError> {
    let issued_at = OffsetDateTime::now_utc();
    let expires_at = issued_at + time::Duration::seconds(SESSION_TTL_SECONDS);
    let mut nonce = [0_u8; SESSION_NONCE_BYTES];
    getrandom(&mut nonce).map_err(AuthError::Randomness)?;
    let username = URL_SAFE_NO_PAD.encode(admin.username.as_bytes());

    let payload = format!(
        "{}.{}.{}.{}.{}",
        SESSION_VERSION,
        issued_at.unix_timestamp(),
        expires_at.unix_timestamp(),
        username,
        URL_SAFE_NO_PAD.encode(nonce)
    );
    let signature = sign_payload(session, &payload)?;

    Ok(format!("{payload}.{signature}"))
}

pub fn create_admin_session_cookie(
    admin: &AdminConfig,
    session: &SessionConfig,
) -> Result<String, AuthError> {
    let token = create_admin_session_token(admin, session)?;

    Ok(format!(
        "{}={}; Path=/admin; Max-Age={}; HttpOnly; SameSite=Lax; Secure",
        ADMIN_SESSION_COOKIE, token, SESSION_TTL_SECONDS
    ))
}

pub fn clear_admin_session_cookie() -> String {
    format!(
        "{}=; Path=/admin; Max-Age=0; HttpOnly; SameSite=Lax; Secure",
        ADMIN_SESSION_COOKIE
    )
}

pub fn verify_admin_session_token(
    admin: &AdminConfig,
    session: &SessionConfig,
    token: &str,
) -> Result<AdminSession, AuthError> {
    let mut parts = token.rsplitn(2, '.');
    let signature = parts.next().ok_or(AuthError::InvalidSession)?;
    let payload = parts.next().ok_or(AuthError::InvalidSession)?;
    let expected_signature = sign_payload(session, payload)?;

    if !constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()) {
        return Err(AuthError::InvalidSession);
    }

    let mut payload_parts = payload.split('.');
    let version = payload_parts.next().ok_or(AuthError::InvalidSession)?;
    let issued_at = parse_timestamp(payload_parts.next())?;
    let expires_at = parse_timestamp(payload_parts.next())?;
    let username = parse_username(payload_parts.next())?;
    let nonce = payload_parts.next().ok_or(AuthError::InvalidSession)?;

    if version != SESSION_VERSION || payload_parts.next().is_some() {
        return Err(AuthError::InvalidSession);
    }

    if !constant_time_eq(username.as_bytes(), admin.username.as_bytes()) {
        return Err(AuthError::InvalidSession);
    }

    URL_SAFE_NO_PAD
        .decode(nonce)
        .map_err(|_| AuthError::InvalidSession)?;

    if expires_at <= OffsetDateTime::now_utc() {
        return Err(AuthError::ExpiredSession);
    }

    Ok(AdminSession {
        username,
        issued_at,
        expires_at,
    })
}

pub fn verify_admin_cookie_header(
    admin: &AdminConfig,
    session: &SessionConfig,
    cookie_header: &str,
) -> Result<AdminSession, AuthError> {
    let token = extract_cookie_value(cookie_header, ADMIN_SESSION_COOKIE)
        .ok_or(AuthError::InvalidSession)?;

    verify_admin_session_token(admin, session, &token)
}

#[cfg(feature = "ssr")]
pub async fn require_admin_session(
    axum::extract::State(config): axum::extract::State<crate::config::AppConfig>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::{
        http::header,
        response::{IntoResponse, Redirect},
    };

    if !is_protected_admin_path(request.uri().path()) {
        return next.run(request).await;
    }

    let has_session = request
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie| {
            verify_admin_cookie_header(&config.admin, &config.session, cookie).ok()
        })
        .is_some();

    if has_session {
        next.run(request).await
    } else {
        Redirect::to("/admin/login").into_response()
    }
}

pub fn extract_cookie_value(cookie_header: &str, cookie_name: &str) -> Option<String> {
    cookie_header.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        if name == cookie_name {
            Some(value.to_owned())
        } else {
            None
        }
    })
}

fn sign_payload(session: &SessionConfig, payload: &str) -> Result<String, AuthError> {
    let mut mac = HmacSha256::new_from_slice(session.secret.as_bytes())
        .map_err(|_| AuthError::Signing)?;
    mac.update(payload.as_bytes());

    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn parse_timestamp(value: Option<&str>) -> Result<OffsetDateTime, AuthError> {
    let timestamp = value
        .ok_or(AuthError::InvalidSession)?
        .parse::<i64>()
        .map_err(|_| AuthError::InvalidSession)?;

    OffsetDateTime::from_unix_timestamp(timestamp).map_err(|_| AuthError::InvalidSession)
}

fn is_protected_admin_path(path: &str) -> bool {
    (path == "/admin" || path.starts_with("/admin/")) && path != "/admin/login"
}

fn parse_username(value: Option<&str>) -> Result<String, AuthError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value.ok_or(AuthError::InvalidSession)?)
        .map_err(|_| AuthError::InvalidSession)?;

    String::from_utf8(bytes).map_err(|_| AuthError::InvalidSession)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && bool::from(left.ct_eq(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_config() -> AdminConfig {
        AdminConfig {
            username: "editor".to_owned(),
            password: "correct-password".to_owned(),
        }
    }

    fn session_config() -> SessionConfig {
        SessionConfig {
            secret: "test-session-secret-with-more-than-32-bytes".to_owned(),
        }
    }

    #[test]
    fn validate_admin_credentials_accepts_exact_match() {
        let result = validate_admin_credentials(&admin_config(), "editor", "correct-password");

        assert!(result.is_ok());
    }

    #[test]
    fn validate_admin_credentials_rejects_wrong_password() {
        let result = validate_admin_credentials(&admin_config(), "editor", "wrong-password");

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn admin_session_cookie_round_trips_through_cookie_header() -> Result<(), AuthError> {
        let admin = admin_config();
        let session = session_config();
        let cookie = create_admin_session_cookie(&admin, &session)?;
        let token = extract_cookie_value(&cookie, ADMIN_SESSION_COOKIE)
            .ok_or(AuthError::InvalidSession)?;
        let cookie_header = format!("theme=dark; {ADMIN_SESSION_COOKIE}={token}; other=value");

        let verified = verify_admin_cookie_header(&admin, &session, &cookie_header)?;

        assert_eq!(verified.username, admin.username);
        assert!(verified.expires_at > verified.issued_at);

        Ok(())
    }

    #[test]
    fn admin_session_rejects_tampered_token() -> Result<(), AuthError> {
        let admin = admin_config();
        let session = session_config();
        let mut token = create_admin_session_token(&admin, &session)?;
        token.push('x');

        let result = verify_admin_session_token(&admin, &session, &token);

        assert!(matches!(result, Err(AuthError::InvalidSession)));

        Ok(())
    }
}
