use super::SendError;

pub trait UnboundedChannelBuilder<E>
where
    E: 'static,
{
    type Sender: UnboundedSender<E>;
    type Receiver: UnboundedReceiver<E>;

    fn chan(&self) -> (Self::Sender, Self::Receiver);
}

pub trait UnboundedSender<E>: 'static
where
    E: 'static,
{
    type Error: SendError<E>;

    fn send(&self, event: E) -> Result<(), Self::Error>;
}

pub trait UnboundedReceiver<E>: 'static + Send + Sync
where
    E: 'static,
{
    fn recv(&mut self) -> impl Future<Output = Option<E>> + Send + Sync;
}

#[async_trait::async_trait]
pub trait UnboundedReceiverDyn<E>: 'static
where
    E: 'static,
{
    async fn recv(&mut self) -> Option<E>;
}

#[async_trait::async_trait]
impl<T, E> UnboundedReceiverDyn<E> for T
where
    T: UnboundedReceiver<E>,
    E: 'static,
{
    async fn recv(&mut self) -> Option<E> {
        UnboundedReceiver::recv(self).await
    }
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {

    use super::*;
    pub struct UnboundedChannelBuilderTokio {}

    impl crate::provider::Provider for UnboundedChannelBuilderTokio {
        fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            Ok(Self {})
        }
    }

    impl<E> UnboundedChannelBuilder<E> for UnboundedChannelBuilderTokio
    where
        E: 'static,
        E: Send + Sync,
    {
        type Sender = UnboundedSenderTokio<E>;
        type Receiver = UnboundedReceiverTokio<E>;

        fn chan(&self) -> (Self::Sender, Self::Receiver) {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (
                UnboundedSenderTokio { sender: tx },
                UnboundedReceiverTokio { receiver: rx },
            )
        }
    }

    pub struct UnboundedSenderTokio<E> {
        sender: tokio::sync::mpsc::UnboundedSender<E>,
    }

    impl<E> Clone for UnboundedSenderTokio<E> {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
            }
        }
    }

    pub struct UnboundedReceiverTokio<E> {
        receiver: tokio::sync::mpsc::UnboundedReceiver<E>,
    }

    impl<E> SendError<E> for tokio::sync::mpsc::error::SendError<E> {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> UnboundedSender<E> for UnboundedSenderTokio<E>
    where
        E: 'static,
    {
        type Error = tokio::sync::mpsc::error::SendError<E>;

        #[inline]
        fn send(&self, event: E) -> Result<(), Self::Error> {
            self.sender.send(event)
        }
    }

    impl<E> UnboundedReceiver<E> for UnboundedReceiverTokio<E>
    where
        E: 'static + Send + Sync,
    {
        async fn recv(&mut self) -> Option<E> {
            self.receiver.recv().await
        }
    }
}
