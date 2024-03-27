use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap, cell::{RefCell, Ref, RefMut},
};



/// A struct holding arbitrary resources.
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
impl Resources {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: HashMap::default(),
        }
    }

    pub fn add<T: 'static>(&mut self, value: T) {
        self.resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(value)));
    }

    /// Immutably gets a value of type `T` from this resource collection.
    ///
    /// # Panics
    /// This method will panic if there is no value of type `T` present.
    pub fn get<T: 'static>(&self) -> Ref<T> {
        Ref::map(
            self.resources
                .get(&TypeId::of::<T>())
                .unwrap_or_else(|| {
                    panic!(
                        "Tried to get type {} which is not present in Resources",
                        type_name::<T>()
                    )
                })
                .try_borrow().unwrap_or_else(|_| panic!("{} already borrowed", type_name::<T>())),
            |v| unsafe { v.downcast_ref_unchecked() }, // SAFETY: We assert at insertion time that the value is of this type.
        )
    }

    /// Mutably gets a value of type `T` from this resource collection.
    ///
    /// # Panics
    /// This method will panic if there is no value of type `T` present.
    pub fn get_mut<T: 'static>(&self) -> RefMut<T> {
        RefMut::map(
            self.resources
                .get(&TypeId::of::<T>())
                .unwrap_or_else(|| {
                    panic!(
                        "Tried to get type {} which is not present in Resources",
                        type_name::<T>()
                    )
                })
                .borrow_mut(),
            |v| unsafe { v.downcast_mut_unchecked() }, // SAFETY: We assert at insertion time that the value is of this type.
        )
    }
}
