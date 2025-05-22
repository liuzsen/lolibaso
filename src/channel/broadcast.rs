use std::any::Any;

use super::SendError;

pub trait BroadcastAsyncChannelBuilder<E>
where
    E: 'static + Clone,
{
    type Sender: BroadcastAsyncSender<E>;
    type Receiver: BroadcastAsyncReceiver<E>;

    fn chan(&self) -> (Self::Sender, Self::Receiver);
}

pub trait BroadcastAsyncSender<E>: 'static
where
    E: 'static + Clone,
{
    type Error: SendError<E>;
    type Receiver: BroadcastAsyncReceiver<E>;

    fn send(&self, value: E) -> Result<(), Self::Error>;

    fn subscribe(&self) -> Self::Receiver;
}

pub trait BroadcastAsyncReceiver<E>: 'static + Send + Sync
where
    E: 'static + Clone,
{
    fn recv(&mut self) -> impl Future<Output = Option<E>> + Send + Sync;
}

#[async_trait::async_trait]
pub trait BroadcastReceiverAsyncDyn<E>: 'static + Any
where
    E: 'static + Clone,
{
    async fn recv(&mut self) -> Option<E>;
}

#[async_trait::async_trait]
impl<E, T> BroadcastReceiverAsyncDyn<E> for T
where
    E: 'static + Clone,
    T: BroadcastAsyncReceiver<E>,
{
    async fn recv(&mut self) -> Option<E> {
        BroadcastAsyncReceiver::recv(self).await
    }
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use std::{fmt::Debug, usize};

    use tokio::sync::broadcast;

    use crate::provider::Provider;

    use super::*;

    pub struct ChanBuilder {}

    impl<E> BroadcastAsyncChannelBuilder<E> for ChanBuilder
    where
        E: 'static + Debug + Clone + Send + Sync,
    {
        type Sender = BroadcastSender<E>;
        type Receiver = BroadcastReceiver<E>;

        fn chan(&self) -> (Self::Sender, Self::Receiver) {
            let sender = BroadcastSender::new();
            let receiver = sender.subscribe();
            (sender, receiver)
        }
    }

    pub struct BroadcastSender<E> {
        sender: broadcast::Sender<E>,
    }

    impl<E> Clone for BroadcastSender<E> {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
            }
        }
    }

    impl<E> BroadcastSender<E>
    where
        E: Clone,
    {
        pub fn new() -> Self {
            let (sender, _) = broadcast::channel(usize::MAX);
            Self { sender }
        }
    }

    pub struct BroadcastReceiver<E> {
        receiver: broadcast::Receiver<E>,
    }

    impl<E> Provider for BroadcastSender<E>
    where
        E: 'static + Clone,
    {
        fn build(ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            if let Some(sender) = ctx.get::<BroadcastSender<E>>() {
                return Ok(sender.clone());
            }

            let sender = BroadcastSender::new();
            ctx.insert(sender.clone());

            Ok(sender)
        }
    }

    impl<E> SendError<E> for broadcast::error::SendError<E>
    where
        E: 'static + Debug,
    {
        fn unsent_item(self) -> E {
            self.0
        }
    }

    impl<E> BroadcastAsyncSender<E> for BroadcastSender<E>
    where
        E: 'static + Debug + Clone + Send + Sync,
    {
        type Error = broadcast::error::SendError<E>;
        type Receiver = BroadcastReceiver<E>;

        fn send(&self, value: E) -> Result<(), Self::Error> {
            self.sender.send(value)?;
            Ok(())
        }

        fn subscribe(&self) -> Self::Receiver {
            BroadcastReceiver {
                receiver: self.sender.subscribe(),
            }
        }
    }

    impl<E> BroadcastAsyncReceiver<E> for BroadcastReceiver<E>
    where
        E: 'static + Clone + Send + Sync,
    {
        async fn recv(&mut self) -> Option<E> {
            let recv = self.receiver.recv();
            let res = recv.await;
            match res {
                Ok(v) => Some(v),
                Err(err) => match err {
                    broadcast::error::RecvError::Closed => None,
                    broadcast::error::RecvError::Lagged(count) => {
                        panic!("Broadcast channel lagged by {}. This is a bug.", count);
                    }
                },
            }
        }
    }
}
