use super::{
    SendError,
    broadcast::{BroadcastReceiver, BroadcastReceiverDyn, BroadcastSender, BroadcastSenderDyn},
    unbounded::{UnboundedReceiver, UnboundedReceiverDyn, UnboundedSender},
};

pub trait DuplexChanBuilder<C, E>: Send + Sync + 'static {
    type Server: DuplexChanServer<Event = E, Command = C>;
    type Client: DuplexChanClient<Event = E, Command = C>;

    fn chan(&self) -> (Self::Server, Self::Client);
}

pub trait DuplexChanServer: Send + Sync + Sized + 'static {
    type Event: 'static + Clone;
    type Command: 'static;
    type Error: SendError<Self::Event>;

    fn send(&self, msg: Self::Event) -> Result<(), Self::Error>;

    fn receive(&mut self) -> impl Future<Output = Option<Self::Command>> + Send + Sync;

    fn split(
        self,
    ) -> (
        impl BroadcastSender<Self::Event, Error = Self::Error>,
        impl UnboundedReceiver<Self::Command>,
    )
    where
        Self::Command: Clone;

    fn client(&self) -> impl DuplexChanClient<Command = Self::Command, Event = Self::Event>;

    fn to_dyn(
        self,
    ) -> Box<
        dyn DuplexChanServerDyn<Event = Self::Event, Command = Self::Command, Error = Self::Error>,
    > {
        Box::new(self)
    }
}

pub trait DuplexChanClient: Send + Sync + Sized + 'static + Clone {
    type Command: 'static;
    type Event: 'static;
    type Error: SendError<Self::Command>;

    fn send(&self, msg: Self::Command) -> Result<(), Self::Error>;

    fn receive(&mut self) -> impl Future<Output = Option<Self::Event>> + Send + Sync;

    fn split(
        self,
    ) -> (
        impl UnboundedSender<Self::Command, Error = Self::Error>,
        impl BroadcastReceiver<Self::Event>,
    )
    where
        Self::Event: Clone;

    fn to_dyn(
        self,
    ) -> Box<
        dyn DuplexChanClientDyn<Command = Self::Command, Event = Self::Event, Error = Self::Error>,
    > {
        Box::new(self)
    }
}

#[async_trait::async_trait]
pub trait DuplexChanClientDyn {
    type Command;
    type Event;
    type Error: SendError<Self::Command>;

    fn send(&self, msg: Self::Command) -> Result<(), Self::Error>;

    async fn receive(&mut self) -> Option<Self::Event>;

    fn split(
        self,
    ) -> (
        Box<dyn UnboundedSender<Self::Command, Error = Self::Error>>,
        Box<dyn BroadcastReceiverDyn<Self::Event>>,
    )
    where
        Self::Event: Clone;
}

#[async_trait::async_trait]
pub trait DuplexChanServerDyn {
    type Event: 'static + Clone;
    type Command: 'static;
    type Error: SendError<Self::Event>;

    fn send(&self, msg: Self::Event) -> Result<(), Self::Error>;

    async fn receive(&mut self) -> Option<Self::Command>;

    fn split(
        self,
    ) -> (
        Box<dyn BroadcastSenderDyn<Self::Event, Error = Self::Error>>,
        Box<dyn UnboundedReceiverDyn<Self::Command>>,
    )
    where
        Self::Command: Clone;
}

#[async_trait::async_trait]
impl<T> DuplexChanClientDyn for T
where
    T: DuplexChanClient,
{
    type Command = T::Command;
    type Event = T::Event;
    type Error = T::Error;

    fn send(&self, msg: Self::Command) -> Result<(), Self::Error> {
        DuplexChanClient::send(self, msg)
    }

    async fn receive(&mut self) -> Option<Self::Event> {
        DuplexChanClient::receive(self).await
    }

    fn split(
        self,
    ) -> (
        Box<dyn UnboundedSender<Self::Command, Error = Self::Error>>,
        Box<dyn BroadcastReceiverDyn<Self::Event>>,
    )
    where
        Self::Event: Clone,
    {
        let (sender, receiver) = DuplexChanClient::split(self);
        (Box::new(sender), Box::new(receiver))
    }
}

#[async_trait::async_trait]
impl<T> DuplexChanServerDyn for T
where
    T: DuplexChanServer,
{
    type Event = T::Event;
    type Command = T::Command;
    type Error = T::Error;

    fn send(&self, msg: Self::Event) -> Result<(), Self::Error> {
        DuplexChanServer::send(self, msg)
    }

    async fn receive(&mut self) -> Option<Self::Command> {
        DuplexChanServer::receive(self).await
    }

    fn split(
        self,
    ) -> (
        Box<dyn BroadcastSenderDyn<Self::Event, Error = Self::Error>>,
        Box<dyn UnboundedReceiverDyn<Self::Command>>,
    )
    where
        Self::Command: Clone,
    {
        let (sender, receiver) = DuplexChanServer::split(self);
        (Box::new(sender), Box::new(receiver))
    }
}

#[cfg(feature = "tokio")]
pub mod impl_tokio {
    use std::fmt::Debug;

    use crate::{
        channel::{
            broadcast::{
                BroadcastChanBuilder,
                impl_tokio::{
                    BroadcastChanBuilderTokio, BroadcastReceiverTokio, BroadcastSenderTokio,
                },
            },
            unbounded::{
                UnboundedChannelBuilder,
                impl_tokio::{
                    UnboundedChannelBuilderTokio, UnboundedReceiverTokio, UnboundedSenderTokio,
                },
            },
        },
        provider::Provider,
    };

    use super::*;

    pub struct DuplexChanBuilderTokio {}

    impl Provider for DuplexChanBuilderTokio {
        fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            Ok(Self {})
        }
    }

    impl<C, E> DuplexChanBuilder<C, E> for DuplexChanBuilderTokio
    where
        C: Send + Sync + 'static,
        E: Send + Sync + 'static + Clone + Debug,
    {
        type Server = DuplexChanServerTokio<C, E>;
        type Client = DuplexChanClientTokio<C, E>;

        fn chan(&self) -> (Self::Server, Self::Client) {
            let (cmd_tx, cmd_rx) = UnboundedChannelBuilderTokio::new().chan();
            let (event_tx, event_rx) = BroadcastChanBuilderTokio::new().chan();

            (
                DuplexChanServerTokio {
                    sender: event_tx,
                    receiver: cmd_rx,
                    cmd_sender: cmd_tx.clone(),
                },
                DuplexChanClientTokio {
                    sender: cmd_tx,
                    receiver: event_rx,
                },
            )
        }
    }
    pub struct DuplexChanClientTokio<C, E> {
        sender: UnboundedSenderTokio<C>,
        receiver: BroadcastReceiverTokio<E>,
    }

    impl<C, E> Clone for DuplexChanClientTokio<C, E>
    where
        E: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
                receiver: self.receiver.clone(),
            }
        }
    }

    impl<S, R> DuplexChanClient for DuplexChanClientTokio<S, R>
    where
        S: Send + Sync + 'static,
        R: Send + Sync + 'static + Clone,
    {
        type Command = S;
        type Event = R;

        type Error = tokio::sync::mpsc::error::SendError<S>;

        fn send(&self, msg: Self::Command) -> Result<(), Self::Error> {
            self.sender.send(msg)
        }

        async fn receive(&mut self) -> Option<Self::Event> {
            BroadcastReceiver::recv(&mut self.receiver).await
        }

        fn split(
            self,
        ) -> (
            impl UnboundedSender<Self::Command, Error = Self::Error>,
            impl BroadcastReceiver<Self::Event>,
        )
        where
            Self::Event: Clone,
        {
            (self.sender, self.receiver)
        }
    }

    pub struct DuplexChanServerTokio<C, E> {
        sender: BroadcastSenderTokio<E>,
        receiver: UnboundedReceiverTokio<C>,
        cmd_sender: UnboundedSenderTokio<C>,
    }

    impl<C, E> DuplexChanServer for DuplexChanServerTokio<C, E>
    where
        E: Send + Sync + 'static + Clone + std::fmt::Debug,
        C: Send + Sync + 'static,
    {
        type Event = E;

        type Command = C;

        type Error = tokio::sync::broadcast::error::SendError<E>;

        fn send(&self, msg: Self::Event) -> Result<(), Self::Error> {
            BroadcastSender::send(&self.sender, msg)
        }

        fn receive(&mut self) -> impl Future<Output = Option<Self::Command>> + Send + Sync {
            UnboundedReceiver::recv(&mut self.receiver)
        }

        fn split(
            self,
        ) -> (
            impl BroadcastSender<Self::Event, Error = Self::Error>,
            impl UnboundedReceiver<Self::Command>,
        )
        where
            Self::Command: Clone,
        {
            (self.sender, self.receiver)
        }

        fn client(&self) -> impl DuplexChanClient<Command = Self::Command, Event = Self::Event> {
            DuplexChanClientTokio {
                sender: self.cmd_sender.clone(),
                receiver: self.sender.subscribe(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DuplexChanClientDyn;

    #[allow(dead_code)]
    fn check_dyn_box<T, S, R, E>(t: T)
    where
        T: DuplexChanClientDyn<Command = S, Event = R, Error = E>,
    {
        let _: Box<dyn DuplexChanClientDyn<Command = S, Event = R, Error = E>> = Box::new(t);
    }
}
