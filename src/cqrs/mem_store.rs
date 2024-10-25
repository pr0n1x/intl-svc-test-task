use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use crate::cqrs::store::StoredEventList;
use super::{Aggregate, store::{EventStore, EventStoreError}};

pub struct MemEventStore<A: Aggregate> {
    // it's not necessary to use RwLock and Arc instead on Rc,
    // but let's imagine we are working in async/multithreading environment
    evs: Arc<RwLock<HashMap<A::Id, StoredEventList<A>>>>,
}

impl<A: Aggregate> MemEventStore<A> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { evs: Arc::new(RwLock::new(HashMap::new())) }
    }
}

fn map_locking_err<E: Error>(_: E) -> EventStoreError {
    EventStoreError::StorageError("MemStorage RwLock had been poisoned".into())
}

impl<A: Aggregate> EventStore<A> for MemEventStore<A> {

    fn fetch(&self, aggregate_id: &A::IdRef) -> Result<StoredEventList<A>, EventStoreError> {
        let events_map = self.evs.read().map_err(map_locking_err)?;
        let events = match events_map.get(aggregate_id) {
            Some(v) => v.clone(),
            None => return Err(EventStoreError::AggregateIsNotExist),
        };
        if events.is_empty() {
            return Err(EventStoreError::AggregateIsNotExist)
        }
        Ok(events)
    }

    fn is_exist(&self, aggregate_id: &<A as Aggregate>::IdRef) -> Result<bool, EventStoreError> {
        let events_map = self.evs.read().map_err(map_locking_err)?;
        Ok(events_map.contains_key(aggregate_id))
    }

    fn commit(&self, event_list: StoredEventList<A>) -> Result<(), EventStoreError> {
        let mut events_map = self.evs.write().map_err(map_locking_err)?;
        events_map.insert(event_list.aggregate_id().to_owned(), event_list);
        Ok(())
    }

    fn remove(&self, aggregate_id: &A::IdRef) -> Result<StoredEventList<A>, EventStoreError> {
        let events_map_read = self.evs.read().map_err(map_locking_err)?;
        let event_list  = match events_map_read.get(aggregate_id) {
            Some(x) => x.clone(),
            None => return Err(EventStoreError::AggregateIsNotExist),
        };
        drop(events_map_read);
        let mut events_map_write = self.evs.write().map_err(map_locking_err)?;
        events_map_write.remove(aggregate_id);
        Ok(event_list)
    }
}
