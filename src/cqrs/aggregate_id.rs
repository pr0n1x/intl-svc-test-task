pub trait AggregateIdContract:
    Clone // also Sized
    + ToString 
    + Into<String> 
    + AsRef<Self::BorrowedAggregateId>
    // + AsRef<str> // commented to avoid an ambiguity on type inference 
    + Eq + PartialEq<Self::BorrowedAggregateId>
    + std::hash::Hash 
    + std::ops::Deref<Target = Self::BorrowedAggregateId>
    + std::borrow::Borrow<Self::BorrowedAggregateId>
    + IsEmptyAggregateId
    + 'static
{
    type BorrowedAggregateId: AggregateIdRefContract<OwnedAggregateId = Self> + ?Sized;
}


pub trait AggregateIdRefContract:
    AsRef<str>
    + ToString
    + ToOwned<Owned = Self::OwnedAggregateId>
    + Eq
    + PartialEq<Self::Owned>
    + std::hash::Hash
    // + ?Sized // traits are ?Szied by default
{
    type OwnedAggregateId: AggregateIdContract<BorrowedAggregateId = Self> + Sized;
}

pub trait IsEmptyAggregateId {
    fn is_empty(&self) -> bool;
}

impl<S: core::convert::AsRef<str>> IsEmptyAggregateId for S {
    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

impl AggregateIdContract for String { type BorrowedAggregateId = str; }
impl AggregateIdRefContract for str { type OwnedAggregateId = String; }
// impl AggregateIdContract for usize { type BorrowedAggregateId = usize; }
// impl AggregateIdRefContract for usize { type OwnedAggregateId = usize; }

// string_based_type!(AggregateId, AggregateIdRef);
// impl AggregateIdContract<AggregateIdRef> for AggregateId {}
// impl AggregateIdRefContract<AggregateId> for AggregateIdRef {}
