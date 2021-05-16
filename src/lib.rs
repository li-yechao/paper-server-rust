pub mod auth;
pub mod paper;
pub mod user;

pub use id::Id;
pub use pagination::{OrderBy, OrderDirection, Pagination, PaginationList};
pub use result::{Error, ErrorKind, Result};

mod id {
    use std::{
        fmt::{self, Display, Formatter},
        marker::PhantomData,
    };

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct Id<T>(String, PhantomData<T>);

    impl<T> Display for Id<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<T> Serialize for Id<T> {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.0)
        }
    }

    impl<'de, T> Deserialize<'de> for Id<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer).map(|x| Self(x, PhantomData))
        }
    }

    impl<T, F> From<F> for Id<T>
    where
        F: Into<String>,
    {
        fn from(s: F) -> Self {
            Self(s.into(), PhantomData)
        }
    }

    impl<T> AsRef<str> for Id<T> {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }
}

mod pagination {
    pub enum Pagination<T> {
        After {
            after: Option<T>,
            skip: Option<u64>,
            first: u64,
        },
        Before {
            before: Option<T>,
            skip: Option<u64>,
            last: u64,
        },
    }

    pub struct PaginationList<T> {
        pub list: Vec<T>,
        pub total: u64,
        pub has_next_page: bool,
    }

    pub enum OrderDirection {
        Asc,

        Desc,
    }

    pub struct OrderBy<T> {
        pub field: T,

        pub direction: OrderDirection,
    }
}

mod result {
    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::ToString)]
    #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
    pub enum ErrorKind {
        Unaurhorized,
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

    impl Error {
        pub fn unauthorized<T: Into<Option<String>>>(message: T) -> Self {
            Self {
                kind: ErrorKind::Unaurhorized,
                message: message.into(),
            }
        }

        pub fn forbidden<T: Into<Option<String>>>(message: T) -> Self {
            Self {
                kind: ErrorKind::Forbidden,
                message: message.into(),
            }
        }

        pub fn not_found<T: Into<Option<String>>>(message: T) -> Self {
            Self {
                kind: ErrorKind::NotFound,
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
}
