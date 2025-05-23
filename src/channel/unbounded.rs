use super::SendError;

pub trait UnboundedChannelBuilderAsync<E>
where
    E: 'static,
{
    type Sender: UnboundedSenderAsync<E>;
    type Receiver: UnboundedReceiverAsync<E>;

    fn chan(&self) -> (Self::Sender, Self::Receiver);
}

pub trait UnboundedSenderAsync<E>: 'static
where
    E: 'static,
{
    type Error: SendError<E>;

    fn send(&self, event: E) -> Result<(), Self::Error>;
}

pub trait UnboundedReceiverAsync<E>: 'static + Send + Sync
where
    E: 'static,
{
    fn recv(&mut self) -> impl Future<Output = Option<E>> + Send + Sync;
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use super::*;

    pub struct UnboundedSender<E> {
        sender: tokio::sync::mpsc::UnboundedSender<E>,
    }

    pub struct UnboundedReceiver<E> {
        receiver: tokio::sync::mpsc::UnboundedReceiver<E>,
    }

    impl<E> SendError<E> for tokio::sync::mpsc::error::SendError<E> {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> UnboundedSenderAsync<E> for UnboundedSender<E>
    where
        E: 'static,
    {
        type Error = tokio::sync::mpsc::error::SendError<E>;

        #[inline]
        fn send(&self, event: E) -> Result<(), Self::Error> {
            self.sender.send(event)
        }
    }

    impl<E> UnboundedReceiverAsync<E> for UnboundedReceiver<E>
    where
        E: 'static + Send + Sync,
    {
        async fn recv(&mut self) -> Option<E> {
            self.receiver.recv().await
        }
    }
}
