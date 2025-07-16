use super::SendError;

pub trait BroadcastChanBuilder<E>: Send + Sync + 'static
where
    E: 'static + Clone,
{
    type Sender: BroadcastSender<E>;
    type Receiver: BroadcastReceiver<E>;

    fn chan(&self) -> (Self::Sender, Self::Receiver) {
        self.chan_with_capacity(1024)
    }

    fn chan_with_capacity(&self, capacity: usize) -> (Self::Sender, Self::Receiver);
}

pub trait BroadcastSender<E>: 'static + Sized + Send + Sync
where
    E: 'static + Clone,
{
    type Error: SendError<E>;
    type Receiver: BroadcastReceiver<E>;

    fn send(&self, value: E) -> Result<(), Self::Error>;

    fn subscribe(&self) -> Self::Receiver;

    fn boxed(self) -> Box<dyn BroadcastSenderDyn<E, Error = Self::Error>> {
        Box::new(self)
    }
}

pub trait BroadcastSenderDyn<E>: 'static + Send + Sync
where
    E: 'static + Clone,
{
    type Error: SendError<E>;

    fn send(&self, value: E) -> Result<(), Self::Error>;

    fn subscribe(&self) -> Box<dyn BroadcastReceiverDyn<E>>;
}

#[async_trait::async_trait]
impl<T, E> BroadcastSenderDyn<E> for T
where
    T: BroadcastSender<E>,
    E: 'static + Clone,
{
    type Error = T::Error;

    fn send(&self, value: E) -> Result<(), Self::Error> {
        BroadcastSender::send(self, value)
    }

    fn subscribe(&self) -> Box<dyn BroadcastReceiverDyn<E>> {
        Box::new(BroadcastSender::subscribe(self))
    }
}

pub trait BroadcastReceiver<E>: 'static + Send + Sync
where
    E: 'static + Clone,
{
    fn recv(&mut self) -> impl Future<Output = Result<Option<E>, Lagged>> + Send + Sync;
}

#[derive(Debug, Clone, Copy)]
pub struct Lagged(pub usize);

impl std::fmt::Display for Lagged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Broadcast channel lagged by {}", self.0)
    }
}

impl std::error::Error for Lagged {}

#[async_trait::async_trait]
pub trait BroadcastReceiverDyn<E>: 'static + Send + Sync
where
    E: 'static + Clone,
{
    async fn recv(&mut self) -> Result<Option<E>, Lagged>;
}

#[async_trait::async_trait]
impl<E, T> BroadcastReceiverDyn<E> for T
where
    E: 'static + Clone,
    T: BroadcastReceiver<E>,
{
    async fn recv(&mut self) -> Result<Option<E>, Lagged> {
        BroadcastReceiver::recv(self).await
    }
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use std::fmt::Debug;

    use tokio::sync::broadcast;

    use crate::provider::Provider;

    use super::*;

    pub struct BroadcastChanBuilderTokio {}

    impl BroadcastChanBuilderTokio {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Provider for BroadcastChanBuilderTokio {
        fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            Ok(Self {})
        }
    }

    impl<E> BroadcastChanBuilder<E> for BroadcastChanBuilderTokio
    where
        E: 'static + Debug + Clone + Send + Sync,
    {
        type Sender = BroadcastSenderTokio<E>;
        type Receiver = BroadcastReceiverTokio<E>;

        fn chan_with_capacity(&self, capacity: usize) -> (Self::Sender, Self::Receiver) {
            let (sender, receiver) = broadcast::channel(capacity);
            (
                BroadcastSenderTokio { sender },
                BroadcastReceiverTokio { receiver },
            )
        }
    }

    pub struct BroadcastSenderTokio<E> {
        sender: broadcast::Sender<E>,
    }

    impl<E> Clone for BroadcastSenderTokio<E> {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
            }
        }
    }

    impl<E> BroadcastSenderTokio<E>
    where
        E: Clone,
    {
        pub fn subscribe(&self) -> BroadcastReceiverTokio<E> {
            BroadcastReceiverTokio {
                receiver: self.sender.subscribe(),
            }
        }
    }

    pub struct BroadcastReceiverTokio<E> {
        receiver: broadcast::Receiver<E>,
    }

    impl<E: Clone> Clone for BroadcastReceiverTokio<E> {
        fn clone(&self) -> Self {
            Self {
                receiver: self.receiver.resubscribe(),
            }
        }
    }

    impl<E> SendError<E> for broadcast::error::SendError<E>
    where
        E: 'static + Debug + Send + Sync,
    {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> BroadcastSender<E> for BroadcastSenderTokio<E>
    where
        E: 'static + Debug + Clone + Send + Sync,
    {
        type Error = broadcast::error::SendError<E>;
        type Receiver = BroadcastReceiverTokio<E>;

        fn send(&self, value: E) -> Result<(), Self::Error> {
            self.sender.send(value)?;
            Ok(())
        }

        fn subscribe(&self) -> Self::Receiver {
            BroadcastReceiverTokio {
                receiver: self.sender.subscribe(),
            }
        }
    }

    impl<E> BroadcastReceiver<E> for BroadcastReceiverTokio<E>
    where
        E: 'static + Clone + Send + Sync,
    {
        async fn recv(&mut self) -> Result<Option<E>, Lagged> {
            let recv = self.receiver.recv();
            let res = recv.await;
            match res {
                Ok(v) => Ok(Some(v)),
                Err(err) => match err {
                    broadcast::error::RecvError::Closed => Ok(None),
                    broadcast::error::RecvError::Lagged(count) => Err(Lagged(count as usize)),
                },
            }
        }
    }
}
