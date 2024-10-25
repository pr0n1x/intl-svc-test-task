#[allow(dead_code)]
pub trait OwnedContract:
    Clone
    + AsRef<Self::Borrowed>
    + std::ops::Deref<Target = Self::Borrowed>
    + std::borrow::Borrow<Self::Borrowed>
    + 'static
{
    type Borrowed: ToOwned<Owned = Self> + ?Sized;
}
