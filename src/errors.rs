use std::sync::PoisonError;

/// Custom error type for this crate
#[derive(thiserror::Error, Debug)]
pub enum MyErrors {
    #[error("Failed to lock mutex: {0}")]
    MutexLockError(String),
}

impl MyErrors {
    pub fn from_poison_error<T>(e: PoisonError<T>) -> Self {
        MyErrors::MutexLockError(e.to_string())
    }
}
