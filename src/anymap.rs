use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct AnyMap {
    data: HashMap<TypeId, Box<dyn Any + 'static>>,
}

impl AnyMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AnyMap {
    /// Retrieve the value stored in the map for the type `T`, if it exists.
    pub fn find<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref())
    }

    /// Retrieve a mutable reference to the value stored in the map for the type `T`, if it exists.
    pub fn find_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut())
    }

    /// Set the value contained in the map for the type `T`.
    /// This will override any previous value stored.
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.data
            .insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any + 'static>);
    }

    /// Remove the value for the type `T` if it existed.
    pub fn remove<T: 'static>(&mut self) {
        self.data.remove(&TypeId::of::<T>());
    }
}

pub struct AnyMapIter<'a> {
    iter: std::collections::hash_map::Iter<'a, TypeId, Box<dyn Any + 'static>>,
}

impl<'a> IntoIterator for &'a AnyMap {
    type Item = (&'a TypeId, &'a Box<dyn Any + 'static>);
    type IntoIter = AnyMapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnyMapIter {
            iter: self.data.iter(),
        }
    }
}

impl<'a> Iterator for AnyMapIter<'a> {
    type Item = (&'a TypeId, &'a Box<dyn Any + 'static>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::AnyMap;

    struct EntryA {
        pub value: u32,
    }

    struct EntryB {
        pub message: String,
    }

    #[test]
    fn anymap() {
        let mut anymap = AnyMap::new();
        anymap.insert(EntryA { value: 3 });
        assert_eq!(anymap.find::<EntryA>().unwrap().value, 3);

        if let Some(entry) = anymap.find_mut::<EntryA>() {
            entry.value = 10;
        }
        assert_eq!(anymap.find::<EntryA>().unwrap().value, 10);

        anymap.insert(EntryB {
            message: "Hi!".to_string(),
        });
        assert_eq!(anymap.find::<EntryB>().unwrap().message, "Hi!");

        anymap.insert(EntryA { value: 4 });
        assert_eq!(anymap.find::<EntryA>().unwrap().value, 4);

        anymap.remove::<EntryB>();
        assert!(anymap.find::<EntryB>().is_none());
    }

    #[test]
    fn anymap_iter() {
        let mut anymap = AnyMap::default();
        anymap.insert(EntryA { value: 3 });
        anymap.insert(EntryB {
            message: "Hi!".to_string(),
        });

        for (type_id, _value) in &anymap {
            println!("TypeId: {:?}", type_id);
        }
    }
}
