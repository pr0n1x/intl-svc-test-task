use crate::OwnedContract;

use super::{Aggregate, IsEmptyAggregateId};

pub type EventIndex = u64;

#[derive(Debug)]
pub struct StoredEvent<A: Aggregate> {
    aggregate_id: A::Id,
    index: EventIndex,
    event: A::Event,
}

#[derive(Clone, Default)]
pub struct StoredEventRawList<A: Aggregate>(Vec<StoredEvent<A>>);
pub struct StoredEventRefList<A: Aggregate>([StoredEvent<A>]);

#[derive(Clone)]
pub struct StoredEventList<A: Aggregate>(StoredEventRawList<A>);

#[derive(Clone, Default)]
pub struct Snapshot<A: Aggregate> {
    aggregate: A,
    index: EventIndex,
}

pub trait EventStore<A: Aggregate> {
    fn fetch(&self, aggregate_id: &A::IdRef) -> Result<StoredEventList<A>, EventStoreError>;
    fn is_exist(&self, aggregate_id: &A::IdRef) -> Result<bool, EventStoreError>;
    fn commit(&self, state: StoredEventList<A>) -> Result<(), EventStoreError>;
    fn remove(&self, aggregate_id: &A::IdRef) -> Result<StoredEventList<A>, EventStoreError>;
}

impl<A: Aggregate> StoredEvent<A> {
    pub fn aggregate_id(&self) -> &A::IdRef {
        &self.aggregate_id
    }
}

impl<A: Aggregate> Snapshot<A> {
    pub fn aggregate(&self) -> &A {
        &self.aggregate
    }

    pub fn to_aggregate(&self) -> A {
        self.aggregate.clone()
    }

    pub fn into_aggregate(self) -> A {
        self.aggregate
    }

    pub fn index(&self) -> EventIndex {
        self.index
    }
}

impl<A: Aggregate> Clone for StoredEvent<A> {
    fn clone(&self) -> Self {
        return Self {
            aggregate_id: self.aggregate_id.clone(),
            index: self.index,
            event: self.event.clone(),
        }
    }
}

impl<A: Aggregate> StoredEventRefList<A> {
    fn new(s: &[StoredEvent<A>]) -> &Self {
        // SAFETY: layout of StoredEventsRef<A> exactly the same as [StoredEvent<A>]
        unsafe { &*(s as *const [StoredEvent<A>] as *const Self) }
    }
}

impl<A: Aggregate> StoredEventRawList<A> {
    pub fn new() -> Self { Self(Vec::new()) }
    
    fn as_slice(&self) -> &StoredEventRefList<A> {
        StoredEventRefList::<A>::new(self.0.as_ref())
    }

    fn append_unchecked(&mut self, aggregate_id: A::Id, event: A::Event) -> StoredEvent<A> {
        let stored_event = StoredEvent {
            aggregate_id, index: self.0.len() as u64, event,
        };
        self.0.push(stored_event.clone());

        stored_event
    }

    fn initial_aggregate_id(&self, maybe_initial_event: &A::Event) -> A::Id {
        if !self.0.is_empty() {
            return self.0[0].aggregate_id.clone();
        }
        let mut created_aggregate = A::default();
        created_aggregate.apply(maybe_initial_event.clone());
        created_aggregate.aggregate_id().to_owned()
    }

    pub fn aggregate_id(&self) -> Option<&A::IdRef> {
        match self.0.is_empty() {
            true => None,
            false => Some(self.aggregate_id_unchecked()),
        }
    }

    fn aggregate_id_unchecked(&self) -> &A::IdRef {
        &self.0[0].aggregate_id
    }

    pub fn append(&mut self, event: A::Event) -> Result<StoredEvent<A>, EventStoreError> {
        let aggregate_id = self.initial_aggregate_id(&event);
        if aggregate_id.is_empty() {
            return Err(EventStoreError::InvalidInitialEvent);
        }
        Ok(self.append_unchecked(aggregate_id, event))
    }

    pub fn append_all(mut self, event_list: &[A::Event]) -> Result<StoredEventList<A>, EventStoreError> {
        match event_list {
            [first, ..] => {
                let aggregate_id: <A as Aggregate>::Id = self.initial_aggregate_id(first);
                if aggregate_id.is_empty() {
                    return Err(EventStoreError::InvalidInitialEvent);
                }
                for event in event_list {
                    self.append_unchecked(aggregate_id.clone(), event.clone());
                }
                Ok(StoredEventList::<A>(self))
            }
            _ => Err(EventStoreError::EmptyEventList),
        }
    }

    pub fn snapshot(&self) -> Option<Snapshot<A>> {
        let events_count = self.0.len();
        if events_count < 1 {
            return None;
        }
        Some(self.snapshot_unchecked())
    }

    fn snapshot_unchecked(&self) -> Snapshot<A> {
        let mut aggregate = A::default();
        for event in &self.0 {
            aggregate.apply(event.event.clone());
        }
        Snapshot { aggregate, index: (self.0.len() as EventIndex) - 1 }
    }

    pub fn snapshot_at(&self, index: EventIndex) -> Option<Snapshot<A>> {
        let events_count = self.0.len();
        if events_count < 1 || (events_count as EventIndex - 1) < index {
            return None;
        }
        Some(self.snapshot_at_unchecked(index))
    }

    fn snapshot_at_unchecked(&self, index: EventIndex) -> Snapshot<A> {
        let mut aggregate = A::default();
        for event in &self.0 {
            aggregate.apply(event.event.clone());
            if event.index == index { break }
        }
        Snapshot { aggregate, index }
    }

    pub fn check_consistency(&self) -> Result<(), EventStoreError> {
        if self.0.is_empty() {
            return Ok(())
        }
        let aggregate_id = self.0[0].aggregate_id.as_ref();
        let mut event_index: EventIndex = 0;
        for event in self.0.iter() {
            if event.aggregate_id.eq(aggregate_id) {
                return Err(EventStoreError::InconsistentEventAggregateId)
            }
            if event.index != event_index {
                 return Err(EventStoreError::InconsistentEventIndex)
            }
            event_index += 1;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn not_empty(self) -> Option<StoredEventList<A>> {
        match self.0.is_empty() {
            true => None,
            false => Some(StoredEventList(self)),
        }
    }
}

impl<A: Aggregate> StoredEventList<A> {
    pub fn new(event_list: &[A::Event]) -> Result<StoredEventList<A>, EventStoreError> {
        StoredEventRawList::new().append_all(event_list)
    }

    pub fn aggregate_id(&self) -> &A::IdRef {
        self.0.aggregate_id_unchecked()
    }
    pub fn snapshot(&self) -> Snapshot<A> {
        self.0.snapshot_unchecked()
    }
    pub fn snapshot_at(&self, index: EventIndex) -> Snapshot<A> {
        self.0.snapshot_at_unchecked(index)
    }

    pub fn append(&mut self, event: A::Event) -> StoredEvent<A> {
        self.0.append_unchecked(self.aggregate_id().to_owned(), event)
    }

    pub fn append_all(mut self, event_list: &[A::Event]) -> StoredEventList<A> {
        let aggregate_id = self.aggregate_id().to_owned();
        for event in event_list {
            self.0.append_unchecked(aggregate_id.clone(), event.clone());
        }
        self
    }

    pub fn raw(self) -> StoredEventRawList<A> {
        self.0
    }
}

impl<A: Aggregate + 'static> OwnedContract for StoredEventRawList<A> {
    type Borrowed = StoredEventRefList<A>;
}

impl<A: Aggregate> AsRef<StoredEventRefList<A>> for StoredEventRawList<A> {
    fn as_ref(&self) -> &StoredEventRefList<A> {
        self.as_slice()
    }
}
impl<A: Aggregate> AsRef<StoredEventRefList<A>> for StoredEventList<A> {
    fn as_ref(&self) -> &StoredEventRefList<A> {
        self.0.as_slice()
    }
}
impl<A: Aggregate> std::ops::Deref for StoredEventRawList<A> {
    type Target = StoredEventRefList<A>;
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<A: Aggregate> std::ops::Deref for StoredEventList<A> {
    type Target = StoredEventRawList<A>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A: Aggregate> core::borrow::Borrow<StoredEventRefList<A>> for StoredEventRawList<A> {
    fn borrow(&self) -> &StoredEventRefList<A> {
        self.as_slice()
    }
}
impl<A: Aggregate> core::borrow::Borrow<StoredEventRefList<A>> for StoredEventList<A> {
    fn borrow(&self) -> &StoredEventRefList<A> {
        self.0.as_slice()
    }
}
impl<A: Aggregate> ToOwned for StoredEventRefList<A> {
    type Owned = StoredEventRawList<A>;
    fn to_owned(&self) -> Self::Owned {
        StoredEventRawList::<A>(self.0.to_owned())
    }
}

impl<A: Aggregate> AsRef<[StoredEvent<A>]> for StoredEventRefList<A> {
    fn as_ref(&self) -> &[StoredEvent<A>] {
        &self.0
    }
}

pub enum EventStoreError {
    InvalidInitialEvent,
    AggregateIsNotExist,
    InconsistentEventAggregateId,
    InconsistentEventIndex,
    EmptyEventList,
    StorageError(Box<dyn core::error::Error + Send + Sync + 'static>)
}

impl core::fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInitialEvent => write!(f, "invalid initial event (aggregate_id is empty after applying it)"),
            Self::AggregateIsNotExist => write!(f, "aggregate does not exist"),
            Self::InconsistentEventAggregateId => write!(f, "inconsistent event aggregate id"),
            Self::InconsistentEventIndex => write!(f, "inconsistent event index number"),
            Self::EmptyEventList => write!(f, "empty event list"),
            Self::StorageError(e) => write!(f, "event storage error: {}", e),
        }
    }
}

impl core::fmt::Debug for EventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageError(e) => write!(f, "event storage error: {:?}", e),
            _ => write!(f, "{}", self),
        }
    }
}

impl core::error::Error for EventStoreError {}
