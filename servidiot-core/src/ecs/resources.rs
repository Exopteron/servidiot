use std::any::{type_name, Any, TypeId};

use ahash::AHashMap;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, MappedRwLockWriteGuard, RwLockWriteGuard};
use thiserror::Error;
#[derive(Default)]
pub struct Resources {
    objects: AHashMap<TypeId, RwLock<Box<dyn Any>>>,
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("resource not present: {0}")]
    ResourceNotPresent(String),
}
pub type ResourceResult<T> = std::result::Result<T, ResourceError>;

impl Resources {
    /// Insert a resource.
    pub fn insert<T: 'static>(&mut self, value: T) -> Option<T> {
        self.objects
            .insert(TypeId::of::<T>(), RwLock::new(Box::new(value)))
            .map(|v| *v.into_inner().downcast::<T>().unwrap())
    }

    /// Remove a resource.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.objects
            .remove(&TypeId::of::<T>())
            .map(|v| *v.into_inner().downcast::<T>().unwrap())
    }

    /// Retrieve a resource for reading.
    pub fn get<T: 'static>(&self) -> ResourceResult<MappedRwLockReadGuard<T>> {
        let resource = self
            .objects
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ResourceError::ResourceNotPresent(type_name::<T>().to_string()))?;
        Ok(RwLockReadGuard::map(resource.read(), |f| {
            f.downcast_ref().unwrap()
        }))
    }

    /// Retrieve a resource for writing.
    pub fn get_mut<T: 'static>(&self) -> ResourceResult<MappedRwLockWriteGuard<T>> {
        let resource = self
            .objects
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ResourceError::ResourceNotPresent(type_name::<T>().to_string()))?;
        Ok(RwLockWriteGuard::map(resource.write(), |f| {
            f.downcast_mut().unwrap()
        }))
    }
}
