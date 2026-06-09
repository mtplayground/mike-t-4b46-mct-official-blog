use std::{error::Error, fmt};

const SESSION_SECRET_MIN_BYTES: usize = 32;

#[derive(Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub admin: AdminConfig,
    pub object_storage: ObjectStorageConfig,
    pub session: SessionConfig,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Clone)]
pub struct AdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct ObjectStorageConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub prefix: String,
    pub endpoint: String,
    pub region: String,
    pub force_path_style: bool,
}

impl ObjectStorageConfig {
    pub fn full_key(&self, relative_key: &str) -> String {
        format!("{}{}", self.prefix, relative_key.trim_start_matches('/'))
    }
}

#[derive(Clone)]
pub struct SessionConfig {
    pub secret: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Missing {
        name: &'static str,
        source: std::env::VarError,
    },
    Empty {
        name: &'static str,
    },
    InvalidBool {
        name: &'static str,
        value: String,
    },
    InvalidStoragePrefix {
        name: &'static str,
        value: String,
    },
    WeakSessionSecret {
        name: &'static str,
        min_bytes: usize,
    },
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database: DatabaseConfig::from_env()?,
            admin: AdminConfig::from_env()?,
            object_storage: ObjectStorageConfig::from_env()?,
            session: SessionConfig::from_env()?,
        })
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            url: required_env("DATABASE_URL")?,
        })
    }
}

impl AdminConfig {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            username: required_env("ADMIN_USERNAME")?,
            password: required_env("ADMIN_PASSWORD")?,
        })
    }
}

impl ObjectStorageConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let prefix = required_env("OBJECT_STORAGE_PREFIX")?;
        if !prefix.ends_with('/') {
            return Err(ConfigError::InvalidStoragePrefix {
                name: "OBJECT_STORAGE_PREFIX",
                value: prefix,
            });
        }

        Ok(Self {
            access_key_id: required_env("OBJECT_STORAGE_ACCESS_KEY_ID")?,
            secret_access_key: required_env("OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
            bucket: required_env("OBJECT_STORAGE_BUCKET")?,
            prefix,
            endpoint: required_env("OBJECT_STORAGE_ENDPOINT")?,
            region: required_env("OBJECT_STORAGE_REGION")?,
            force_path_style: bool_env("OBJECT_STORAGE_FORCE_PATH_STYLE")?,
        })
    }
}

impl SessionConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let secret = required_env("SESSION_SECRET")?;
        if secret.len() < SESSION_SECRET_MIN_BYTES {
            return Err(ConfigError::WeakSessionSecret {
                name: "SESSION_SECRET",
                min_bytes: SESSION_SECRET_MIN_BYTES,
            });
        }

        Ok(Self { secret })
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name, .. } => write!(f, "{name} environment variable is not set"),
            Self::Empty { name } => write!(f, "{name} environment variable must not be empty"),
            Self::InvalidBool { name, .. } => {
                write!(f, "{name} must be a boolean value such as true or false")
            }
            Self::InvalidStoragePrefix { name, .. } => {
                write!(f, "{name} must be non-empty and end with a trailing slash")
            }
            Self::WeakSessionSecret { name, min_bytes } => {
                write!(f, "{name} must be at least {min_bytes} bytes long")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Missing { source, .. } => Some(source),
            Self::Empty { .. }
            | Self::InvalidBool { .. }
            | Self::InvalidStoragePrefix { .. }
            | Self::WeakSessionSecret { .. } => None,
        }
    }
}

fn required_env(name: &'static str) -> Result<String, ConfigError> {
    let value = std::env::var(name).map_err(|source| ConfigError::Missing { name, source })?;
    if value.is_empty() {
        return Err(ConfigError::Empty { name });
    }

    Ok(value)
}

fn bool_env(name: &'static str) -> Result<bool, ConfigError> {
    let value = required_env(name)?;
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::InvalidBool { name, value }),
    }
}
