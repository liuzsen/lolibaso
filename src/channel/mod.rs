use std::fmt;

pub mod broadcast;
pub mod duplex;
pub mod unbounded;

pub trait SendError<T>: std::error::Error {
    fn unsent_item(self) -> T;
}

pub type SendErrorDyn<T> = Box<dyn SendError<T>>;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ChanClosed<T>(pub T);

impl<T> fmt::Debug for ChanClosed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for ChanClosed<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T> std::error::Error for ChanClosed<T> {}
impl<T> SendError<T> for ChanClosed<T> {
    fn unsent_item(self) -> T {
        self.0
    }
}
