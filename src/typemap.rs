use std::any::{Any, TypeId};
use std::collections::HashMap as Map;

type TypeValue = Box<dyn Any + Send + 'static>;

#[inline]
fn type_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[derive(Default, Debug)]
pub struct TypeMap {
    map: Map<TypeId, TypeValue>,
}

impl TypeMap {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: Map::with_capacity(capacity),
        }
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
    #[inline]
    pub fn insert<T: Send + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .insert(type_of::<T>(), Box::new(val))
            .and_then(|v| v.downcast().map(|v| *v).ok())
    }

    #[inline]
    pub fn contains<T: Send + 'static>(&self) -> bool {
        self.map.get(&type_of::<T>()).is_some()
    }
    #[inline]

    pub fn get<T: Send + 'static>(&self) -> Option<&T> {
        self.map.get(&type_of::<T>()).and_then(|v| v.downcast_ref())
    }
    #[inline]

    pub fn get_mut<T: Send + 'static>(&mut self) -> Option<&mut T> {
        self.map.get_mut(&type_of::<T>()).and_then(|v| v.downcast_mut())
    }
    #[inline]
    pub fn remove<T: Send + 'static>(&mut self) -> Option<T> {
        self.map.remove(&type_of::<T>()).and_then(|v| v.downcast().map(|v| *v).ok())
    }
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

// cargo tr typemap::test_typemap
#[test]
fn test_typemap() {
    let mut map = TypeMap::new();
    assert!(map.get::<()>().is_none());

    map.insert(());
    assert!(map.get::<()>().is_some());
    assert!(map.get_mut::<()>().is_some());

    assert!(map.remove::<()>().is_some());
    assert!(map.get::<()>().is_none());
    assert!(map.get_mut::<()>().is_none());

    let str = "xyz".to_owned();
    map.insert(str.clone());
    assert!(map.get::<String>().is_some());
    assert!(map.get_mut::<String>().is_some());

    assert!(map.remove::<String>().is_some());
    assert!(map.get::<String>().is_none());
    assert!(map.get_mut::<String>().is_none());

    assert!(map.is_empty());
    map.insert("fx");
    assert!(map.len() == 1);

    std::thread::spawn(move || println!("{:?}", map)).join().unwrap();
}
