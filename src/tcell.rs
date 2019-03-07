use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

lazy_static! {
    static ref SINGLETON_CHECK: Mutex<HashSet<TypeId>> = Mutex::new(HashSet::new());
}

/// Borrowing-owner of zero or more [`TCell`](struct.TCell.html)
/// instances.
///
/// See [crate documentation](index.html).
pub struct TCellOwner<Q: 'static> {
    typ: PhantomData<Q>,
}

impl<Q: 'static> Drop for TCellOwner<Q> {
    fn drop(&mut self) {
        SINGLETON_CHECK.lock().unwrap().remove(&TypeId::of::<Q>());
    }
}

impl<Q: 'static> TCellOwner<Q> {
    /// Create the singleton owner instance.  There may only be one
    /// instance of this type at any time for each different marker
    /// type `Q`.  This call panics if another instance is created.
    /// This may be used for creating many `TCell` instances.
    pub fn new() -> Self {
        assert!(
            SINGLETON_CHECK.lock().unwrap().insert(TypeId::of::<Q>()),
            "Illegal to create two TCellOwner instances with the same marker type parameter"
        );
        Self { typ: PhantomData }
    }

    /// Borrow contents of a `TCell` immutably.  Many `TCell`
    /// instances can be borrowed immutably at the same time from the
    /// same owner.
    #[inline]
    pub fn get<'a, T>(&'a self, qc: &'a TCell<Q, T>) -> &'a T {
        unsafe { &*qc.value.get() }
    }

    /// Borrow contents of a `TCell` mutably.  Only one `TCell` at a
    /// time can be borrowed from the owner using this call.  The
    /// returned reference must go out of scope before another can be
    /// borrowed.
    #[inline]
    pub fn get_mut<'a, T>(&'a mut self, qc: &'a TCell<Q, T>) -> &'a mut T {
        unsafe { &mut *qc.value.get() }
    }

    /// Borrow contents of two `TCell` instances mutably.  Panics if
    /// the two `TCell` instances point to the same memory.
    #[inline]
    pub fn get_mut2<'a, T, U>(
        &'a mut self,
        qc1: &'a TCell<Q, T>,
        qc2: &'a TCell<Q, U>,
    ) -> (&'a mut T, &'a mut U) {
        assert!(
            qc1 as *const _ as usize != qc2 as *const _ as usize,
            "Illegal to borrow same TCell twice with get_mut2()"
        );
        unsafe { (&mut *qc1.value.get(), &mut *qc2.value.get()) }
    }

    /// Borrow contents of three `TCell` instances mutably.  Panics if
    /// any pair of `TCell` instances point to the same memory.
    #[inline]
    pub fn get_mut3<'a, T, U, V>(
        &'a mut self,
        qc1: &'a TCell<Q, T>,
        qc2: &'a TCell<Q, U>,
        qc3: &'a TCell<Q, V>,
    ) -> (&'a mut T, &'a mut U, &'a mut V) {
        assert!(
            (qc1 as *const _ as usize != qc2 as *const _ as usize)
                && (qc2 as *const _ as usize != qc3 as *const _ as usize)
                && (qc3 as *const _ as usize != qc1 as *const _ as usize),
            "Illegal to borrow same TCell twice with get_mut3()"
        );
        unsafe {
            (
                &mut *qc1.value.get(),
                &mut *qc2.value.get(),
                &mut *qc3.value.get(),
            )
        }
    }
}

/// Cell whose contents is owned (for borrowing purposes) by a
/// [`TCellOwner`].
///
/// To borrow from this cell, use the borrowing calls on the
/// [`TCellOwner`] instance that was used to create it.  See [crate
/// documentation](index.html).
///
/// [`TCellOwner`]: struct.TCellOwner.html
pub struct TCell<Q, T> {
    owner: PhantomData<Q>,
    value: UnsafeCell<T>,
}

impl<Q, T> TCell<Q, T> {
    /// Create a new `TCell` owned for borrowing purposes by the given
    /// `TCellOwner<Q>`
    #[inline]
    pub const fn new(_owner: &TCellOwner<Q>, value: T) -> TCell<Q, T> {
        TCell {
            owner: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TCell, TCellOwner};
    #[test]
    #[should_panic]
    fn tcell_singleton_1() {
        struct Marker;
        let _owner1 = TCellOwner::<Marker>::new();
        let _owner2 = TCellOwner::<Marker>::new(); // Panic here
    }

    #[test]
    fn tcell_singleton_2() {
        struct Marker;
        let owner1 = TCellOwner::<Marker>::new();
        drop(owner1);
        let _owner2 = TCellOwner::<Marker>::new();
    }

    #[test]
    fn tcell_singleton_3() {
        struct Marker1;
        struct Marker2;
        let _owner1 = TCellOwner::<Marker1>::new();
        let _owner2 = TCellOwner::<Marker2>::new();
    }

    #[test]
    fn tcell() {
        struct Marker;
        type ACellOwner = TCellOwner<Marker>;
        type ACell<T> = TCell<Marker, T>;
        let mut owner = ACellOwner::new();
        let c1 = ACell::new(&owner, 100u32);
        let c2 = ACell::new(&owner, 200u32);
        (*owner.get_mut(&c1)) += 1;
        (*owner.get_mut(&c2)) += 2;
        let c1ref = owner.get(&c1);
        let c2ref = owner.get(&c2);
        let total = *c1ref + *c2ref;
        assert_eq!(total, 303);
    }
}