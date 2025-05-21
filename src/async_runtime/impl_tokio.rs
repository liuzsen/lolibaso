pub struct AsyncRuntime {}

impl super::JoinError for tokio::task::JoinError {
    fn is_panic(&self) -> bool {
        self.is_panic()
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled()
    }
}

impl<T> super::AsyncTaskHandle<T> for tokio::task::JoinHandle<T> {
    type Error = tokio::task::JoinError;

    fn abort(&self) {
        self.abort();
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}

impl super::AsyncRuntime for AsyncRuntime {
    type AsyncTaskHandle<T> = tokio::task::JoinHandle<T>;

    async fn select<A, B>(&self, f1: A, f2: B) -> super::Either<A::Output, B::Output>
    where
        A: Future,
        B: Future,
    {
        tokio::select! {
            r1 = f1 => {
                super::Either::Left(r1)
            }
            r1 = f2 => {
                super::Either::Right(r1)
            }
        }
    }

    async fn select3<F1, F2, F3>(
        &self,
        f1: F1,
        f2: F2,
        f3: F3,
    ) -> super::Either3<F1::Output, F2::Output, F3::Output>
    where
        F1: Future,
        F2: Future,
        F3: Future,
    {
        tokio::select! {
            r = f1 => {
                super::Either3::R1(r)
            }
            r = f2 => {
                super::Either3::R2(r)
            }
            r = f3 => {
                super::Either3::R3(r)
            }
        }
    }

    fn spawn<F>(&self, future: F) -> Self::AsyncTaskHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::task::spawn(future)
    }

    fn spawn_local<F>(&self, future: F) -> Self::AsyncTaskHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static,
    {
        tokio::task::spawn_local(future)
    }

    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(future)
    }

    async fn join<F1, F2>(&self, f1: F1, f2: F2) -> (F1::Output, F2::Output)
    where
        F1: Future + 'static,
        F2: Future + 'static,
        F1::Output: Send + 'static,
        F2::Output: Send + 'static,
    {
        tokio::join!(f1, f2)
    }

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
        F3::Output: Send + 'static,
    {
        tokio::join!(f1, f2, f3)
    }
}
