use super::SendError;

pub trait UnboundedAsyncChannelBuilder<E>
where
    E: 'static,
{
    type Sender: UnboundedAsyncSender<E>;
    type Receiver: UnboundedAsyncReceiver<E>;

    fn chan(&self) -> (Self::Sender, Self::Receiver);
}

pub trait UnboundedAsyncSender<E>: 'static
where
    E: 'static,
{
    type Error: SendError<E>;

    fn send(&self, event: E) -> Result<(), Self::Error>;
}

pub trait UnboundedAsyncReceiver<E>: 'static + Send + Sync
where
    E: 'static,
{
    fn recv(&mut self) -> impl Future<Output = Option<E>> + Send + Sync;
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use super::*;

    pub struct Sender<E> {
        sender: tokio::sync::mpsc::UnboundedSender<E>,
    }

    pub struct Receiver<E> {
        receiver: tokio::sync::mpsc::UnboundedReceiver<E>,
    }

    impl<E> SendError<E> for tokio::sync::mpsc::error::SendError<E> {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> UnboundedAsyncSender<E> for Sender<E>
    where
        E: 'static,
    {
        type Error = tokio::sync::mpsc::error::SendError<E>;

        #[inline]
        fn send(&self, event: E) -> Result<(), Self::Error> {
            self.sender.send(event)
        }
    }

    impl<E> UnboundedAsyncReceiver<E> for Receiver<E>
    where
        E: 'static + Send + Sync,
    {
        async fn recv(&mut self) -> Option<E> {
            self.receiver.recv().await
        }
    }
}
