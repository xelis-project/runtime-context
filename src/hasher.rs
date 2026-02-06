use std::{
    any::TypeId,
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

/// A hasher for `TypeId`s that takes advantage of its known characteristics.
#[derive(Debug, Default)]
pub struct TypeIdHasher(u64);

/// A `HashMap` optimized for `TypeId` keys.
pub type TypeMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

impl Hasher for TypeIdHasher {
    fn write(&mut self, _: &[u8]) {
        unimplemented!("This TypeIdHasher can only handle u64s")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
