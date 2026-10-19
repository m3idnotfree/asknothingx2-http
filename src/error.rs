use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct Error(BoxError);

impl Error {
    pub(crate) fn new(err: impl Into<BoxError>) -> Self {
        let err = err.into();
        match err.downcast::<Error>() {
            Ok(err) => *err,
            Err(err) => Self(err),
        }
    }

    pub fn is<E: StdError + 'static>(&self) -> bool {
        self.0.is::<E>()
    }

    pub fn downcast_ref<E: StdError + 'static>(&self) -> Option<&E> {
        self.0.downcast_ref()
    }

    pub fn into_inner(self) -> BoxError {
        self.0
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("asknothingx2_http::Error")
            .field(&self.0)
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
