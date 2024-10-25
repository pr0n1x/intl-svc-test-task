#[macro_export]
macro_rules! string_based_type {
    (@inner, $owned_type:ident, $ref_type:ident) => {
        impl $ref_type {
            pub fn new<S: core::convert::AsRef<str> + ?Sized>(s: &S) -> &$ref_type {
                Self::from_str(s.as_ref())
            }
            pub const fn from_str(s: &str) -> &$ref_type {
                // SAFETY: layout of $ref_type exactly the same as str
                unsafe { &*(s as *const str as *const $ref_type) }
            }
            pub const fn len(&self) -> usize { self.0.len() }
            pub const fn is_empty(&self) -> bool { self.0.is_empty() }
            pub const fn as_str(&self) -> &str { &self.0 }
        }
        impl $owned_type {
            pub fn new<S: core::convert::AsRef<str> + ?Sized>(s: &S) -> $owned_type {
                $owned_type(s.as_ref().to_owned())
            }
            pub fn len(&self) -> usize { self.0.len() }
            pub fn is_empty(&self) -> bool { self.0.is_empty() }
            pub fn as_str(&self) -> &str { &self.0 }
            pub fn borrow(&self) -> &$ref_type { $ref_type::from_str(self.0.as_str()) }
        }
        impl From<String> for $owned_type {
            fn from(s: String) -> $owned_type {
                $owned_type(s.into())
            }
        }
        impl From<$owned_type> for String {
            fn from(s: $owned_type) -> String {
                s.0
            }
        }
        impl ToString for $owned_type {
            fn to_string(&self) -> String { self.0.to_string() }
        }
        impl ToString for $ref_type {
            fn to_string(&self) -> String { self.0.to_string() }
        }
        impl core::convert::AsRef<str> for $ref_type {
            fn as_ref(&self) -> &str { &self.0 }
        }
        impl core::convert::AsRef<$ref_type> for str {
            fn as_ref(&self) -> &$ref_type { $ref_type::new(self) }
        }
        impl core::convert::AsRef<str> for $owned_type {
            fn as_ref(&self) -> &str { &self.0 }
        }
        impl core::convert::AsRef<$ref_type> for $owned_type {
            fn as_ref(&self) -> &$ref_type { self }
        }
        impl std::ops::Deref for $owned_type {
            type Target = $ref_type;
            fn deref(&self) -> &$ref_type {
                $ref_type::from_str(&self.0)
            }
        }
        impl std::borrow::ToOwned for $ref_type {
            type Owned = $owned_type;
            fn to_owned(&self) -> $owned_type { $owned_type(self.0.to_owned()) }
            fn clone_into(&self, target: &mut $owned_type) { self.0.clone_into(&mut target.0); }
        }
        impl std::borrow::Borrow<$ref_type> for $owned_type {
            fn borrow(&self) -> &$ref_type {
                $owned_type::borrow(self)
            }
        }
        impl<S: core::convert::AsRef<str> + ?Sized> From<&S> for $owned_type {
            fn from(s: &S) -> $owned_type {
                $owned_type::new(s.as_ref())
            }
        }
        impl PartialEq<$ref_type> for $owned_type {
            fn eq(&self, other: &$ref_type) -> bool {
                self.0.eq(other.as_ref())
            }
        }
        impl PartialEq<$owned_type> for $ref_type {
            fn eq(&self, other: &$owned_type) -> bool {
                self.0.eq(other.as_str())
            }
        }
    };

    ($owned_type:ident, $ref_type:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $owned_type(String);
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $ref_type(str);
        string_based_type!(@inner, $owned_type, $ref_type);
    };

    ($owned_type:ident exists, $ref_type:ident) => {
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct $ref_type(str);
        string_based_type!(@inner, $owned_type, $ref_type);
    };
}
