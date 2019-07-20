#![feature(cfg_target_has_atomic)]

use std::sync::atomic::{
    self, Ordering,
};


/// Types that support atomic operations on the current platform.
pub trait Atom {
    type Impl: AtomicImpl;
    fn pack(self) -> <Self::Impl as AtomicImpl>::Inner;
    fn unpack(src: <Self::Impl as AtomicImpl>::Inner) -> Self;
}

pub struct Atomic<T: Atom>(T::Impl);

impl<T: Atom> Atomic<T> {
    pub fn new(v: T) -> Self {
        Self(T::Impl::new(v.pack()))
    }

    // fn get_mut(&mut self) -> &mut Self::Inner;

    pub fn into_inner(self) -> T {
        T::unpack(self.0.into_inner())
    }
    pub fn load(&self, order: Ordering) -> T {
        T::unpack(self.0.load(order))
    }
    pub fn store(&self, v: T, order: Ordering) {
        self.0.store(v.pack(), order);
    }
    pub fn swap(&self, v: T, order: Ordering) -> T {
        T::unpack(self.0.swap(v.pack(), order))
    }

    pub fn compare_and_swap(&self, current: T, new: T, order: Ordering) -> T {
        T::unpack(self.0.compare_and_swap(current.pack(), new.pack(), order))
    }

    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.0.compare_exchange(current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }

    pub fn compare_exchange_weak(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.0.compare_exchange_weak(current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }
}

impl<T: Atom> Atomic<T>
where
    T::Impl: AtomicLogicImpl,
{
    pub fn fetch_and(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_and(val.pack(), order))
    }
    pub fn fetch_nand(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_nand(val.pack(), order))
    }
    pub fn fetch_or(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_or(val.pack(), order))
    }
    pub fn fetch_xor(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_xor(val.pack(), order))
    }
}



pub trait AtomicImpl {
    type Inner;

    fn new(v: Self::Inner) -> Self;
    fn get_mut(&mut self) -> &mut Self::Inner;
    fn into_inner(self) -> Self::Inner;
    fn load(&self, order: Ordering) -> Self::Inner;
    fn store(&self, v: Self::Inner, order: Ordering);
    fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner;

    fn compare_and_swap(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        order: Ordering,
    ) -> Self::Inner;

    fn compare_exchange(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;

    fn compare_exchange_weak(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;
}

pub trait AtomicLogicImpl: AtomicImpl {
    fn fetch_and(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_nand(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_or(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_xor(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
}

// ===============================================================================================
// ===== Implementations for standard library types
// ===============================================================================================

/// Expands to the `pack` and `unpack` methods implemented as ID function.
macro_rules! id_pack_unpack {
    () => {
        fn pack(self) -> <Self::Impl as AtomicImpl>::Inner {
            self
        }
        fn unpack(src: <Self::Impl as AtomicImpl>::Inner) -> Self {
            src
        }
    };
}

macro_rules! pass_through_methods {
    ($ty:ty) => {
        #[inline(always)]
        fn new(v: Self::Inner) -> Self {
            <$ty>::new(v)
        }
        #[inline(always)]
        fn get_mut(&mut self) -> &mut Self::Inner {
            self.get_mut()
        }
        #[inline(always)]
        fn into_inner(self) -> Self::Inner {
            self.into_inner()
        }
        #[inline(always)]
        fn load(&self, order: Ordering) -> Self::Inner {
            self.load(order)
        }
        #[inline(always)]
        fn store(&self, v: Self::Inner, order: Ordering) {
            self.store(v, order)
        }
        #[inline(always)]
        fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner {
            self.swap(v, order)
        }
        #[inline(always)]
        fn compare_and_swap(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            order: Ordering,
        ) -> Self::Inner {
            self.compare_and_swap(current, new, order)
        }
        #[inline(always)]
        fn compare_exchange(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Inner, Self::Inner> {
            self.compare_exchange(current, new, success, failure)
        }
        #[inline(always)]
        fn compare_exchange_weak(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Inner, Self::Inner> {
            self.compare_exchange_weak(current, new, success, failure)
        }
    };
}

macro_rules! logical_pass_through_methods {
    () => {
        fn fetch_and(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_and(val, order)
        }
        fn fetch_nand(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_nand(val, order)
        }
        fn fetch_or(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_or(val, order)
        }
        fn fetch_xor(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_xor(val, order)
        }
    };
}

#[cfg(target_has_atomic = "ptr")]
impl<T> Atom for *mut T {
    type Impl = atomic::AtomicPtr<T>;
    id_pack_unpack!();
}

#[cfg(target_has_atomic = "ptr")]
impl<T> AtomicImpl for atomic::AtomicPtr<T> {
    type Inner = *mut T;
    pass_through_methods!(atomic::AtomicPtr<T>);
}


macro_rules! impl_std_atomics {
    ($ty:ty, $impl_ty:ident) => {
        impl Atom for $ty {
            type Impl = atomic::$impl_ty;
            id_pack_unpack!();
        }

        impl AtomicImpl for atomic::$impl_ty {
            type Inner = $ty;
            pass_through_methods!(atomic::$impl_ty);
        }

        impl AtomicLogicImpl for atomic::$impl_ty {
            logical_pass_through_methods!();
        }
    };
}

#[cfg(target_has_atomic = "8")] impl_std_atomics!(bool, AtomicBool);
#[cfg(target_has_atomic = "8")] impl_std_atomics!(u8, AtomicU8);
#[cfg(target_has_atomic = "8")] impl_std_atomics!(i8, AtomicI8);
#[cfg(target_has_atomic = "16")] impl_std_atomics!(u16, AtomicU16);
#[cfg(target_has_atomic = "16")] impl_std_atomics!(i16, AtomicI16);
#[cfg(target_has_atomic = "32")] impl_std_atomics!(u32, AtomicU32);
#[cfg(target_has_atomic = "32")] impl_std_atomics!(i32, AtomicI32);
#[cfg(target_has_atomic = "64")] impl_std_atomics!(u64, AtomicU64);
#[cfg(target_has_atomic = "64")] impl_std_atomics!(i64, AtomicI64);
