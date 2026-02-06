use better_any::{Tid, TidExt};

/// Thread-safe variant of `Tid`.
///
/// This trait is automatically implemented for any `Tid` type that is `Send + Sync`.
pub trait ShareableTid<'a>: Tid<'a> + Send + Sync {}

impl<'a, T: Tid<'a> + Send + Sync> ShareableTid<'a> for T {}

/// Stored value variants inside a `Context`.
///
/// Values may be owned, immutably borrowed, or mutably borrowed.
pub enum Data<'ty, 'r> {
    Owned(Box<dyn ShareableTid<'ty>>),
    Borrowed(&'r dyn ShareableTid<'ty>),
    Mut(&'r mut dyn ShareableTid<'ty>),
}

impl<'ty, 'r> Data<'ty, 'r> {
    /// Downcast to a shared reference of the underlying value.
    pub fn downcast_ref<'b, T: Tid<'ty>>(&'b self) -> Option<&'b T> {
        match self {
            Data::Owned(value) => (**value).downcast_ref(),
            Data::Borrowed(value) => (*value).downcast_ref(),
            Data::Mut(value) => (*value).downcast_ref(),
        }
    }

    /// Downcast to a mutable reference of the underlying value.
    pub fn downcast_mut<'b, T: Tid<'ty>>(&'b mut self) -> Option<&'b mut T> {
        match self {
            Data::Owned(value) => (**value).downcast_mut(),
            Data::Mut(value) => (*value).downcast_mut(),
            _ => None,
        }
    }

    /// Convert into an owned value.
    ///
    /// Borrowed values are cloned. Returns `Err(self)` when the type does not match.
    pub fn into_owned<T: Clone + Tid<'ty>>(self) -> Result<T, Self> {
        match self {
            Data::Owned(value) => match value.downcast_box::<T>() {
                Ok(value) => Ok(*value),
                Err(v) => Err(Data::Owned(v)),
            },
            Data::Borrowed(value) => match value.downcast_ref::<T>() {
                Some(value) => Ok(value.clone()),
                None => Err(Data::Borrowed(value)),
            },
            Data::Mut(value) => match value.downcast_ref::<T>() {
                Some(value) => Ok(value.clone()),
                None => Err(Data::Mut(value)),
            },
        }
    }

    /// Take the owned value if present.
    ///
    /// Borrowed variants return `Err(self)`.
    pub fn try_take_owned<T: Tid<'ty>>(self) -> Result<T, Self> {
        match self {
            Data::Owned(value) => match value.downcast_box::<T>() {
                Ok(value) => Ok(*value),
                Err(v) => Err(Data::Owned(v)),
            },
            _ => Err(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use better_any::tid;

    use super::*;

    #[derive(Debug, Clone)]
    struct Test;
    tid!(Test);

    #[test]
    fn test_data_owned() {
        let mut owned = Data::Owned(Box::new(Test));

        let _ = owned.downcast_ref::<Test>().unwrap();
        let _ = owned.downcast_mut::<Test>().unwrap();
        assert!(matches!(owned.into_owned::<Test>(), Ok(Test)));

        let owned = Data::Owned(Box::new(Test));
        assert!(matches!(owned.try_take_owned::<Test>(), Ok(Test)));
    }

    #[test]
    fn test_data_ref() {
        let test = Test;
        let mut borrowed = Data::Borrowed(&test);

        let _ = borrowed.downcast_ref::<Test>().unwrap();
        assert!(borrowed.downcast_mut::<Test>().is_none());
        assert!(matches!(borrowed.into_owned::<Test>(), Ok(Test)));

        let borrowed = Data::Borrowed(&test);
        assert!(matches!(borrowed.try_take_owned::<Test>(), Err(_)));
    }

    #[test]
    fn test_data_mut() {
        let mut test = Test;
        let mut mut_ref = Data::Mut(&mut test);

        let _ = mut_ref.downcast_ref::<Test>().unwrap();
        let _ = mut_ref.downcast_mut::<Test>().unwrap();
        assert!(matches!(mut_ref.into_owned::<Test>(), Ok(Test)));

        let mut_ref = Data::Mut(&mut test);
        assert!(matches!(mut_ref.try_take_owned::<Test>(), Err(_)));
    }

    #[test]
    fn test_into_owned_wrong_type() {
        #[derive(Debug, Clone)]
        struct Other;
        tid!(Other);

        let data = Data::Owned(Box::new(Test));
        assert!(matches!(data.into_owned::<Other>(), Err(_)));
    }
}
