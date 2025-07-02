pub trait GlobalTaskChanStorage<Id, Chan> {
    fn insert(&self, task_id: Id, chan: Chan) -> Option<Chan>;

    fn take_chan(&self, task_id: Id) -> Option<Chan>;

    fn exists(&self, task_id: Id) -> bool;

    fn get_cloned(&self, task_id: Id) -> Option<Chan>
    where
        Chan: Clone;
}

pub mod default_impl {
    use std::{
        any::{Any, TypeId},
        collections::HashMap,
        sync::LazyLock,
    };

    use parking_lot::RwLock;

    use crate::{
        channel::duplex::DuplexChanClient, provider::Provider,
        use_case::long_run::GlobalTaskChanStorage,
    };

    pub struct HashMapTaskChanStorage {
        map: &'static RwLock<HashMap<TypeId, HashMap<String, TaskChanWithTypeName>>>,
    }

    struct TaskChanWithTypeName {
        chan: Box<dyn Any + Send + Sync>,
        type_name: &'static str,
    }

    impl HashMapTaskChanStorage {
        pub fn new() -> Self {
            static GLOBAL_MAP: LazyLock<
                RwLock<HashMap<TypeId, HashMap<String, TaskChanWithTypeName>>>,
            > = LazyLock::new(|| RwLock::new(Default::default()));

            Self { map: &GLOBAL_MAP }
        }
    }

    impl Provider for HashMapTaskChanStorage {
        fn build(_ctx: &mut crate::provider::ProviderContext) -> anyhow::Result<Self> {
            Ok(Self::new())
        }
    }

    impl<Id, Chan> GlobalTaskChanStorage<Id, Chan> for HashMapTaskChanStorage
    where
        Chan: DuplexChanClient,
        Id: ToString + 'static,
    {
        fn insert(&self, task_id: Id, chan: Chan) -> Option<Chan> {
            let mut lock = self.map.write();
            let task_map = lock.entry(task_id.type_id()).or_insert_with(HashMap::new);
            let old = task_map.insert(
                task_id.to_string(),
                TaskChanWithTypeName {
                    chan: Box::new(chan),
                    type_name: std::any::type_name::<Chan>(),
                },
            );
            old.map(TaskChanWithTypeName::cast_to)
        }

        fn take_chan(&self, task_id: Id) -> Option<Chan> {
            let mut lock = self.map.write();
            let task_map = lock.get_mut(&task_id.type_id());
            if let Some(task_map) = task_map {
                let chan = task_map.remove(&task_id.to_string())?;
                Some(chan.cast_to::<Chan>())
            } else {
                None
            }
        }

        fn exists(&self, task_id: Id) -> bool {
            let lock = self.map.read();
            let task_map = lock.get(&task_id.type_id());
            if let Some(task_map) = task_map {
                task_map.contains_key(&task_id.to_string())
            } else {
                false
            }
        }

        fn get_cloned(&self, task_id: Id) -> Option<Chan>
        where
            Chan: Clone,
        {
            let lock = self.map.read();
            let task_map = lock.get(&task_id.type_id());
            if let Some(task_map) = task_map {
                let chan = task_map.get(&task_id.to_string())?;
                Some(chan.cast_to_ref::<Chan>().clone())
            } else {
                None
            }
        }
    }

    impl TaskChanWithTypeName {
        fn cast_to<Chan: 'static>(self) -> Chan {
            match self.chan.downcast::<Chan>() {
                Ok(chan) => *chan,
                Err(_) => {
                    let msg = format!(
                        "Type mismatch. Expected {}, got {}",
                        std::any::type_name::<Chan>(),
                        self.type_name,
                    );
                    panic!("{}", msg);
                }
            }
        }

        fn cast_to_ref<Chan: 'static>(&self) -> &Chan {
            match self.chan.downcast_ref::<Chan>() {
                Some(chan) => chan,
                None => {
                    let msg = format!(
                        "Type mismatch. Expected {}, got {}",
                        std::any::type_name::<Chan>(),
                        self.type_name,
                    );
                    panic!("{}", msg);
                }
            }
        }
    }
}
