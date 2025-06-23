use crate::result::BizResult;

pub trait UseCase: 'static {
    type Input;
    type Output;
    type Error;

    fn execute(
        self,
        input: Self::Input,
    ) -> impl Future<Output = BizResult<Self::Output, Self::Error>> + 'static;
}

pub trait UseCaseWithEvent: UseCase {
    type Event: 'static + Clone;

    fn subscribe(&self) -> impl crate::channel::broadcast::BroadcastReceiver<Self::Event>;
}
