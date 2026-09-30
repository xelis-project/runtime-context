use super::{Data, ShareableTid, TypeMap};
use std::any::TypeId;

/// Runtime context storing values by type.
///
/// The context can store owned values as well as borrowed references (immutable
/// or mutable). Values are keyed by `TypeId` using a specialized hasher for
/// fast lookups.
pub struct Context<'ty, 'r> {
    data: TypeMap<Data<'ty, 'r>>,
}

impl Default for Context<'_, '_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'ty, 'r> Context<'ty, 'r> {
    /// Create a new empty `Context`.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: TypeMap::default(),
        }
    }

    /// Return the number of stored entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Extend the context with another context, consuming it.
    ///
    /// Entries from `other` overwrite existing entries with the same `TypeId`.
    #[inline]
    pub fn extend(&mut self, other: Self) {
        self.data.extend(other.data);
    }

    /// Insert a value into the context without checking the type.
    ///
    /// This is a low-level escape hatch for advanced use-cases.
    #[inline]
    pub fn insert_unchecked(&mut self, key: TypeId, data: Data<'ty, 'r>) {
        self.data.insert(key, data);
    }

    /// Insert a borrowed value into the context.
    #[inline]
    pub fn insert_ref<T: ShareableTid<'ty>>(&mut self, value: &'r T) {
        self.data.insert(T::id(), Data::Borrowed(value));
    }

    /// Insert a mutable reference into the context.
    #[inline]
    pub fn insert_mut<T: ShareableTid<'ty>>(&mut self, value: &'r mut T) {
        self.data.insert(T::id(), Data::Mut(value));
    }

    /// Insert an owned value into the context.
    #[inline]
    pub fn insert<T: ShareableTid<'ty>>(&mut self, value: T) {
        self.data.insert(T::id(), Data::Owned(Box::new(value)));
    }

    /// Get a shared reference to a stored value by type.
    #[inline]
    pub fn get<'b, T: ShareableTid<'ty>>(&'b self) -> Option<&'b T> {
        self.data.get(&T::id()).and_then(|v| v.downcast_ref())
    }

    /// Get a mutable reference to a stored value by type.
    #[inline]
    pub fn get_mut<'b, T: ShareableTid<'ty>>(&'b mut self) -> Option<&'b mut T> {
        self.data.get_mut(&T::id()).and_then(|v| v.downcast_mut())
    }

    /// Get a mutable reference to a stored value, inserting `value` if absent.
    ///
    /// The inserted value is owned by the context. Existing owned values and
    /// mutable references are preserved.
    ///
    /// # Panics
    ///
    /// Panics if the existing entry is an immutable reference or contains a
    /// different type (for example, after using `insert_unchecked`).
    #[inline]
    pub fn get_or_insert<T: ShareableTid<'ty>>(&mut self, value: T) -> &mut T {
        self.get_or_insert_with(|| value)
    }

    /// Get a mutable reference to a stored value, calling `f` to insert if absent.
    ///
    /// The closure is called only when the type has no entry. The inserted value
    /// is owned by the context. Existing owned values and mutable references
    /// are preserved.
    ///
    /// # Panics
    ///
    /// Panics if the existing entry is an immutable reference or contains a
    /// different type (for example, after using `insert_unchecked`).
    #[inline]
    pub fn get_or_insert_with<T: ShareableTid<'ty>, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        self.data
            .entry(T::id())
            .or_insert_with(|| Data::Owned(Box::new(f())))
            .downcast_mut()
            .expect("stored entry cannot be mutably accessed as the requested type")
    }

    /// Get a stored `Data` by `TypeId`.
    #[inline]
    pub fn get_data<'b>(&'b self, id: &TypeId) -> Option<&'b Data<'ty, 'r>> {
        self.data.get(id)
    }

    /// Get a mutable `Data` by `TypeId`.
    #[inline]
    pub fn get_data_mut<'b>(&'b mut self, id: &TypeId) -> Option<&'b mut Data<'ty, 'r>> {
        self.data.get_mut(id)
    }

    /// Get multiple mutable `Data` entries by distinct `TypeId`s.
    #[inline]
    pub fn get_disjoint_mut<'b, const N: usize>(
        &'b mut self,
        keys: [&TypeId; N],
    ) -> [Option<&'b mut Data<'ty, 'r>>; N] {
        self.data.get_disjoint_mut(keys)
    }

    /// Remove an owned value from the context and return it.
    #[inline]
    pub fn take<T: ShareableTid<'ty>>(&mut self) -> Option<T> {
        let id = T::id();
        match self.data.remove(&id) {
            Some(data) => data.try_take_owned::<T>().ok(),
            None => None,
        }
    }

    /// Remove any stored value for the given type and return the raw `Data`.
    #[inline]
    pub fn remove<T: ShareableTid<'ty>>(&mut self) -> Option<Data<'ty, 'r>> {
        self.data.remove(&T::id())
    }

    /// Check if a value of a specific type is present.
    #[inline]
    pub fn contains<T: ShareableTid<'ty>>(&self) -> bool {
        self.data.contains_key(&T::id())
    }

    /// Clear all values from the context.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use better_any::{Tid, tid};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Dummy<'a>(&'a str);
    tid!(Dummy<'_>);

    #[test]
    fn test_context_owned() {
        let dummy = Dummy("Hello, World!");
        let mut context = Context::new();

        context.insert(dummy);
        assert!(matches!(context.get::<Dummy>(), Some(_)));
        assert!(matches!(context.get_mut::<Dummy>(), Some(_)));
        assert_eq!(context.contains::<Dummy>(), true);
    }

    #[test]
    fn test_context_ref() {
        let dummy = Dummy("Hello, World!");
        let mut context = Context::new();

        context.insert_ref(&dummy);
        assert_eq!(context.get::<Dummy>(), Some(&dummy));
        assert_eq!(context.get_mut::<Dummy>(), None);
        assert_eq!(context.contains::<Dummy>(), true);
    }

    #[test]
    fn test_context_mut() {
        let mut dummy = Dummy("Hello, World!");
        let mut context = Context::new();

        context.insert_mut(&mut dummy);
        assert!(matches!(context.get::<Dummy>(), Some(_)));
        assert!(matches!(context.get_mut::<Dummy>(), Some(_)));
        assert_eq!(context.contains::<Dummy>(), true);
    }

    #[test]
    fn test_get_or_insert_absent() {
        let text = String::from("Hello");
        let mut context = Context::new();

        let value = context.get_or_insert(Dummy(&text));
        assert_eq!(value.0, "Hello");
        value.0 = "Updated";

        assert_eq!(context.len(), 1);
        assert_eq!(context.take::<Dummy>(), Some(Dummy("Updated")));
    }

    #[test]
    fn test_get_or_insert_owned() {
        let mut context = Context::new();
        context.insert(Dummy("Existing"));

        let value = context.get_or_insert(Dummy("Replacement"));
        assert_eq!(value.0, "Existing");
        value.0 = "Updated";

        assert_eq!(context.len(), 1);
        assert_eq!(context.get::<Dummy>(), Some(&Dummy("Updated")));
    }

    #[test]
    fn test_get_or_insert_mut() {
        let mut dummy = Dummy("Existing");
        {
            let mut context = Context::new();
            context.insert_mut(&mut dummy);

            let value = context.get_or_insert(Dummy("Replacement"));
            assert_eq!(value.0, "Existing");
            value.0 = "Updated";
        }

        assert_eq!(dummy.0, "Updated");
    }

    #[test]
    fn test_get_or_insert_ref_panics_without_replacing() {
        let dummy = Dummy("Existing");
        let mut context = Context::new();
        context.insert_ref(&dummy);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            context.get_or_insert(Dummy("Replacement"));
        }));

        assert!(result.is_err());
        assert_eq!(context.get::<Dummy>(), Some(&dummy));
        assert!(context.get_mut::<Dummy>().is_none());
    }

    #[test]
    #[should_panic(expected = "stored entry cannot be mutably accessed as the requested type")]
    fn test_get_or_insert_wrong_type() {
        struct Other;
        tid!(Other);

        let mut context = Context::new();
        context.insert_unchecked(Dummy::id(), Data::Owned(Box::new(Other)));
        context.get_or_insert(Dummy("Replacement"));
    }

    #[test]
    fn test_get_or_insert_with_absent() {
        let text = String::from("Hello");
        let dummy = Box::new(Dummy(&text));
        let mut calls = 0;
        let mut context = Context::new();

        let value = context.get_or_insert_with(|| {
            calls += 1;
            *dummy
        });
        assert_eq!(value.0, "Hello");
        value.0 = "Updated";

        assert_eq!(calls, 1);
        assert_eq!(context.take::<Dummy>(), Some(Dummy("Updated")));
    }

    #[test]
    fn test_get_or_insert_with_existing() {
        let mut dummy = Dummy("Borrowed");
        let mut context = Context::new();
        context.insert(Dummy("Owned"));

        let value = context.get_or_insert_with::<Dummy, _>(|| panic!("must not run"));
        assert_eq!(value.0, "Owned");
        value.0 = "Updated owned";
        assert_eq!(context.get::<Dummy>(), Some(&Dummy("Updated owned")));

        context.insert_mut(&mut dummy);
        let value = context.get_or_insert_with::<Dummy, _>(|| panic!("must not run"));
        assert_eq!(value.0, "Borrowed");
        value.0 = "Updated borrowed";
        drop(context);
        assert_eq!(dummy.0, "Updated borrowed");
    }

    #[test]
    fn test_get_or_insert_with_immutable_does_not_call_closure() {
        let dummy = Dummy("Existing");
        let mut context = Context::new();
        context.insert_ref(&dummy);
        let mut called = false;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            context.get_or_insert_with(|| {
                called = true;
                Dummy("Replacement")
            });
        }));

        assert!(result.is_err());
        assert!(!called);
        assert_eq!(context.get::<Dummy>(), Some(&dummy));
    }

    #[test]
    fn test_context_no_immutable_err() {
        let mut dummy = Dummy("Hello, World!");
        {
            let mut context = Context::new();
            context.insert_mut(&mut dummy);
        }

        assert_eq!(dummy.0, "Hello, World!");
    }

    #[test]
    fn test_downcast_to_trait() {
        trait Foo {
            fn foo(&self) -> &str;
        }

        impl Foo for Dummy<'_> {
            fn foo(&self) -> &str {
                self.0
            }
        }

        struct FooWrapper<'a, T: Foo + 'static>(&'a mut T);
        tid! { impl<'a, T: 'static> TidAble<'a> for FooWrapper<'a, T> where T: Foo }

        let mut dummy = Dummy("Hello, World!");
        let mut context = Context::new();
        context.insert(FooWrapper(&mut dummy));

        fn inner_ref_fn<T: Foo + 'static>(context: &Context) {
            let data = context
                .get_data(&FooWrapper::<T>::id())
                .expect("Data not found");
            data.downcast_ref::<FooWrapper<T>>()
                .expect("Downcast failed")
                .0
                .foo();
        }

        inner_ref_fn::<Dummy>(&context);

        fn inner_mut_fn<T: Foo + 'static>(context: &mut Context) {
            let data = context
                .get_data_mut(&FooWrapper::<T>::id())
                .expect("Data not found");
            data.downcast_mut::<FooWrapper<T>>()
                .expect("Downcast failed")
                .0
                .foo();
        }

        inner_mut_fn::<Dummy>(&mut context);
    }

    #[test]
    fn test_take_and_remove() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct TakeMe(u64);
        tid!(TakeMe);

        let mut context = Context::new();
        context.insert(TakeMe(7));

        let owned = context.take::<TakeMe>().unwrap();
        assert_eq!(owned, TakeMe(7));
        assert_eq!(context.contains::<TakeMe>(), false);

        context.insert(TakeMe(9));
        let data = context.remove::<TakeMe>().unwrap();
        assert!(matches!(data.try_take_owned::<TakeMe>(), Ok(TakeMe(9))));
    }

    #[test]
    fn test_get_disjoint_mut() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct A(u8);
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct B(u8);
        tid!(A);
        tid!(B);

        let mut context = Context::new();
        context.insert(A(1));
        context.insert(B(2));

        let [a, b] = context.get_disjoint_mut([&A::id(), &B::id()]);
        let a = a.unwrap().downcast_mut::<A>().unwrap();
        let b = b.unwrap().downcast_mut::<B>().unwrap();

        a.0 += 1;
        b.0 += 2;

        assert_eq!(context.get::<A>().unwrap().0, 2);
        assert_eq!(context.get::<B>().unwrap().0, 4);
    }

    #[test]
    fn test_clear_and_get_data() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct C(i32);
        tid!(C);

        let mut context = Context::new();
        context.insert(C(10));

        let data = context.get_data(&C::id()).unwrap();
        assert_eq!(data.downcast_ref::<C>().unwrap().0, 10);

        context.clear();
        assert_eq!(context.get::<C>(), None);
    }
}
