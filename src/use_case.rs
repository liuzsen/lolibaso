use crate::result::BizResult;

pub trait UseCase {
    type Input;
    type Output;
    type Error;

    async fn execute(self, input: Self::Input) -> BizResult<Self::Output, Self::Error>;
}

pub trait UseCaseWithEvent: UseCase {
    type Event: 'static + Clone;

    fn subscribe(&self) -> impl crate::channel::broadcast::BroadcastAsyncReceiver<Self::Event>;
}
