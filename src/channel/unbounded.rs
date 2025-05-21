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

pub trait UnboundedAsyncReceiver<E>: 'static
where
    E: 'static,
{
    fn recv(&mut self) -> impl Future<Output = anyhow::Result<Option<E>>> + Send + Sync + 'static;
}

#[cfg(feature = "tokio")]
mod impl_tokio {
    use super::*;

    impl<E> SendError<E> for tokio::sync::mpsc::error::SendError<E> {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> UnboundedAsyncSender<E> for tokio::sync::mpsc::UnboundedSender<E>
    where
        E: 'static,
    {
        type Error = tokio::sync::mpsc::error::SendError<E>;

        #[inline]
        fn send(&self, event: E) -> Result<(), Self::Error> {
            self.send(event)
        }
    }
}
