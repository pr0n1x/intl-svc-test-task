pub mod store;
pub mod mem_store;
mod aggregate_id;

pub use aggregate_id::*;

pub trait DomainEvent: Clone + core::fmt::Debug + Sync + Send {
    const EVENT_TYPE: &'static str;
    #[allow(dead_code)]
    fn event_name(&self) -> &'static str;
}

// It also could have been: Default + Send + Sync + Serialize + DeserializeOwned, when using persistent storages
pub trait Aggregate: Default + Clone { 
    type Event: DomainEvent;
    type Id: AggregateIdContract<BorrowedAggregateId = Self::IdRef>;
    type IdRef: AggregateIdRefContract<OwnedAggregateId = Self::Id> + ?Sized;
    
    fn aggregate_type() -> &'static Self::IdRef;
    fn aggregate_id(&self) -> &Self::IdRef;
    fn apply(&mut self, event: Self::Event);
}

#[allow(dead_code)]
pub trait CommandHandler<A: Aggregate> {
    type Error: std::error::Error;
    type Command; // assumed that type is an enum
    type Services; // any dependent services

    fn handle(&self, command: Self::Command, services: Self::Services) -> Result<Vec<A::Event>, Self::Error>;
}
