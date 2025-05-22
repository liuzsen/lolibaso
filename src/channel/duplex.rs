use super::{
    SendError,
    broadcast::{BroadcastAsyncReceiver, BroadcastReceiverAsyncDyn},
    unbounded::UnboundedAsyncSender,
};

pub trait DuplexChan: Send + Sync {
    type SendType: 'static;
    type ReceiveType: 'static;
    type Error: SendError<Self::SendType>;

    fn send(&self, msg: Self::SendType) -> Result<(), Self::Error>;

    fn receive(&mut self) -> impl Future<Output = Option<Self::ReceiveType>> + Send + Sync;

    fn split(
        self,
    ) -> (
        impl UnboundedAsyncSender<Self::SendType, Error = Self::Error>,
        impl BroadcastAsyncReceiver<Self::ReceiveType>,
    )
    where
        Self::ReceiveType: Clone;
}

#[async_trait::async_trait]
pub trait DuplexChanDyn {
    type SendType;
    type ReceiveType;
    type Error: SendError<Self::SendType>;

    fn send(&self, msg: Self::SendType) -> Result<(), Self::Error>;

    async fn receive(&mut self) -> Option<Self::ReceiveType>;

    fn split(
        self,
    ) -> (
        Box<dyn UnboundedAsyncSender<Self::SendType, Error = Self::Error>>,
        Box<dyn BroadcastReceiverAsyncDyn<Self::ReceiveType>>,
    )
    where
        Self::ReceiveType: Clone;
}

#[async_trait::async_trait]
impl<T> DuplexChanDyn for T
where
    T: DuplexChan,
{
    type SendType = T::SendType;
    type ReceiveType = T::ReceiveType;
    type Error = T::Error;

    fn send(&self, msg: Self::SendType) -> Result<(), Self::Error> {
        DuplexChan::send(self, msg)
    }

    async fn receive(&mut self) -> Option<Self::ReceiveType> {
        DuplexChan::receive(self).await
    }

    fn split(
        self,
    ) -> (
        Box<dyn UnboundedAsyncSender<Self::SendType, Error = Self::Error>>,
        Box<dyn BroadcastReceiverAsyncDyn<Self::ReceiveType>>,
    )
    where
        Self::ReceiveType: Clone,
    {
        let (sender, receiver) = DuplexChan::split(self);
        (Box::new(sender), Box::new(receiver))
    }
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use crate::channel::{broadcast::impl_tokio::BroadcastReceiver, unbounded::impl_tokio::Sender};

    use super::*;

    pub struct DuplexChanTokio<S, R> {
        sender: Sender<S>,
        receiver: BroadcastReceiver<R>,
    }

    impl<S, R> DuplexChan for DuplexChanTokio<S, R>
    where
        S: Send + Sync + 'static,
        R: Send + Sync + 'static + Clone,
    {
        type SendType = S;
        type ReceiveType = R;

        type Error = tokio::sync::mpsc::error::SendError<S>;

        fn send(&self, msg: Self::SendType) -> Result<(), Self::Error> {
            self.sender.send(msg)
        }

        async fn receive(&mut self) -> Option<Self::ReceiveType> {
            BroadcastAsyncReceiver::recv(&mut self.receiver).await
        }

        fn split(
            self,
        ) -> (
            impl UnboundedAsyncSender<Self::SendType, Error = Self::Error>,
            impl BroadcastAsyncReceiver<Self::ReceiveType>,
        )
        where
            Self::ReceiveType: Clone,
        {
            (self.sender, self.receiver)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DuplexChanDyn;

    #[allow(dead_code)]
    fn check_dyn_box<T, S, R, E>(t: T)
    where
        T: DuplexChanDyn<SendType = S, ReceiveType = R, Error = E>,
    {
        let _: Box<dyn DuplexChanDyn<SendType = S, ReceiveType = R, Error = E>> = Box::new(t);
    }
}
