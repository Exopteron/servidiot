use std::{any::TypeId, collections::HashMap};



/// A map keyed by types.
#[derive(Clone)]
pub struct TypeMap<T> {
    inner: HashMap<TypeId, T>
}


impl<T> TypeMap<T> {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<K: 'static>(&self) -> Option<&T> {
        self.inner.get(&TypeId::of::<K>())
    }

    pub fn insert<K: 'static>(&mut self, val: T) -> Option<T> {
        self.inner.insert(TypeId::of::<K>(), val)
    }
}

impl<T> Default for TypeMap<T> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeMap;

    #[test]
    fn basic_functionality() {
        let mut v = TypeMap::new();
        v.insert::<String>(24i32);
        v.insert::<f64>(25i32);

        assert_eq!(v.get::<String>(), Some(&24));
        assert_eq!(v.get::<f64>(), Some(&25));
        assert_eq!(v.get::<f32>(), None);
    }
}