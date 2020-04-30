
#[macro_export]
macro_rules! atomic_type {
    ($ty_name:ident,$inner_ty:ty,$atomic_name:ident,$atomic_ty:ty) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
        pub struct $ty_name($inner_ty);

        impl $ty_name {
            pub const fn into(self) -> $inner_ty {
                self.0
            }
            pub const fn from(x: $inner_ty) -> Self {
                Self(x)
            }
        }

        pub struct $atomic_name {
            container: $atomic_ty,
        }

        impl $atomic_name {
            #[allow(dead_code)]
            pub const fn new(id: $ty_name) -> Self {
                Self {
                    container: <$atomic_ty>::new(id.into())
                }
            }
            #[allow(dead_code)]
            pub fn increment(&self) -> $ty_name {
                $ty_name::from(self.container.fetch_add(1, Ordering::Relaxed))
            }
            #[allow(dead_code)]
            pub const fn default() -> Self {
                Self::new($ty_name::from(0))
            }
            #[allow(dead_code)]
            pub fn load(&self, order: ::core::sync::atomic::Ordering) -> $ty_name {
                $ty_name::from(self.container.load(order))
            }
            #[allow(dead_code)]
            pub fn store(&self, val: $ty_name, order: ::core::sync::atomic::Ordering) {
                self.container.store(val.into(), order)
            }
            #[allow(dead_code)]
            pub fn swap(&self, val: $ty_name, order: ::core::sync::atomic::Ordering) -> $ty_name {
                $ty_name::from(self.container.swap(val.into(), order))
            }
            #[allow(dead_code)]
            pub fn compare_and_swap(&self, current: $ty_name, new: $ty_name, order: ::core::sync::atomic::Ordering) -> $ty_name {
                $ty_name::from(self.container.compare_and_swap(current.into(), new.into(), order))
            }
            #[allow(dead_code)]
            pub fn compare_exchange(&self, current: $ty_name, new: $ty_name, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<$ty_name, $ty_name> {
                match self.container.compare_exchange(current.into(), new.into(), success, failure) {
                    Ok(result) => Ok($ty_name::from(result)),
                    Err(result) => Err($ty_name::from(result))
                }
            }
            #[allow(dead_code)]
            pub fn compare_exchange_weak(&self, current: $ty_name, new: $ty_name, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<$ty_name, $ty_name> {
                match self.container.compare_exchange_weak(current.into(), new.into(), success, failure) {
                    Ok(result) => Ok($ty_name::from(result)),
                    Err(result) => Err($ty_name::from(result))
                }
            }
        }
    };
}