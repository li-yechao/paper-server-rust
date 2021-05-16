use juniper::{graphql_value, FieldError, IntoFieldError, ScalarValue};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::ToString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,

    pub message: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.kind.to_string(),
            self.message.as_ref().map(String::as_ref).unwrap_or("None")
        )
    }
}

impl std::error::Error for Error {}

impl From<paper::Error> for Error {
    fn from(e: paper::Error) -> Self {
        Self {
            kind: match e.kind {
                paper::ErrorKind::Unaurhorized => ErrorKind::Unauthorized,
                paper::ErrorKind::Forbidden => ErrorKind::Forbidden,
                paper::ErrorKind::NotFound => ErrorKind::NotFound,
                paper::ErrorKind::Unknown => ErrorKind::Unknown,
            },
            message: e.message,
        }
    }
}

impl Error {
    pub fn unauthorized<T: Into<Option<String>>>(message: T) -> Self {
        Self {
            kind: ErrorKind::Unauthorized,
            message: message.into(),
        }
    }

    pub fn forbidden<T: Into<Option<String>>>(message: T) -> Self {
        Self {
            kind: ErrorKind::Forbidden,
            message: message.into(),
        }
    }

    pub fn unknown<T: Into<Option<String>>>(message: T) -> Self {
        Self {
            kind: ErrorKind::Unknown,
            message: message.into(),
        }
    }
}

impl<S: ScalarValue> IntoFieldError<S> for Error {
    fn into_field_error(self) -> FieldError<S> {
        let kind = self.kind.to_string();

        FieldError::new(
            self.message.as_ref().map(String::as_ref).unwrap_or("None"),
            graphql_value!({
                "type": kind,
            }),
        )
    }
}
