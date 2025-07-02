use crate::channel::duplex::DuplexChanClient;

pub trait GlobalTaskChanStorage<C>
where
    C: DuplexChanClient<Command = Self::Command, Event = Self::Event>,
{
    type TaskId;
    type Command: 'static;
    type Event: 'static;

    fn insert(&self, task_id: Self::TaskId, chan: C) -> Option<C>;

    fn exists(&self, task_id: Self::TaskId) -> bool;

    fn get_cloned(&self, task_id: Self::TaskId) -> Option<C>
    where
        C: Clone;

    fn take_chan(&self, task_id: Self::TaskId) -> Option<C>;
}

pub mod default_impl {
    use std::{collections::HashMap, sync::LazyLock};

    use parking_lot::RwLock;

    use crate::{channel::duplex::DuplexChanClient, provider::Provider};

    use super::GlobalTaskChanStorage;

    pub struct HashMapTaskChanStorage<Id, Chan> {
        map: LazyLock<RwLock<HashMap<Id, Chan>>>,
    }

    impl<I, C> Provider for HashMapTaskChanStorage<I, C>
    where
        I: 'static,
        C: 'static,
    {
        fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            Ok(Self::new_global())
        }
    }

    impl<Id, Chan> HashMapTaskChanStorage<Id, Chan> {
        pub fn new_global() -> Self {
            Self {
                map: LazyLock::new(|| RwLock::new(Default::default())),
            }
        }
    }

    impl<Id, Chan> GlobalTaskChanStorage<Chan> for HashMapTaskChanStorage<Id, Chan>
    where
        Chan: DuplexChanClient,
        Id: Eq + std::hash::Hash,
    {
        type TaskId = Id;
        type Command = Chan::Command;
        type Event = Chan::Event;

        fn insert(&self, task_id: Self::TaskId, chan: Chan) -> Option<Chan> {
            self.map.write().insert(task_id, chan)
        }

        fn exists(&self, task_id: Self::TaskId) -> bool {
            self.map.read().contains_key(&task_id)
        }

        fn take_chan(&self, task_id: Self::TaskId) -> Option<Chan> {
            self.map.write().remove(&task_id)
        }

        fn get_cloned(&self, task_id: Self::TaskId) -> Option<Chan>
        where
            Chan: Clone,
        {
            self.map.read().get(&task_id).cloned()
        }
    }
}
