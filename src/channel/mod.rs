pub mod unbounded;

pub trait SendError<T>: std::error::Error {
    fn unsent_item(self) -> T;
}
