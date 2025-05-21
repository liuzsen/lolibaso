#[cfg(feature = "tokio")]
mod impl_tokio;

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub enum Either3<R1, R2, R3> {
    R1(R1),
    R2(R2),
    R3(R3),
}

pub trait AsyncRuntime {
    type AsyncTaskHandle<T>: AsyncTaskHandle<T>;

    async fn select<A, B>(&self, f1: A, f2: B) -> Either<A::Output, B::Output>
    where
        A: Future,
        B: Future;

    async fn select3<F1, F2, F3>(
        &self,
        f1: F1,
        f2: F2,
        f3: F3,
    ) -> Either3<F1::Output, F2::Output, F3::Output>
    where
        F1: Future,
        F2: Future,
        F3: Future;

    fn spawn<F>(&self, future: F) -> Self::AsyncTaskHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn spawn_local<F>(&self, future: F) -> Self::AsyncTaskHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static;

    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    async fn join<F1, F2>(&self, f1: F1, f2: F2) -> (F1::Output, F2::Output)
    where
        F1: Future + 'static,
        F2: Future + 'static,
        F1::Output: Send + 'static,
        F2::Output: Send + 'static;

    async fn join3<F1, F2, F3>(
        &self,
        f1: F1,
        f2: F2,
        f3: F3,
    ) -> (F1::Output, F2::Output, F3::Output)
    where
        F1: Future + 'static,
        F2: Future + 'static,
        F3: Future + 'static,
        F1::Output: Send + 'static,
        F2::Output: Send + 'static,
        F3::Output: Send + 'static;
}

pub trait AsyncTaskHandle<T>: Future<Output = Result<T, Self::Error>> {
    type Error: JoinError;

    fn abort(&self);

    fn is_finished(&self) -> bool;
}

pub trait JoinError: std::error::Error {
    fn is_panic(&self) -> bool;

    fn is_cancelled(&self) -> bool;
}
