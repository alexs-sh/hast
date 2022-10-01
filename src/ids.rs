use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type Id = u64;

struct IdStorage {
    ids: HashMap<Id, String>,
}

impl IdStorage {
    fn make_hash(value: &str) -> Id {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    pub fn add(&mut self, value: &str) -> Id {
        let hash = IdStorage::make_hash(value);
        self.ids.entry(hash).or_insert_with(|| value.to_owned());
        hash
    }

    fn get_by_value(&self, value: &str) -> Option<Id> {
        let hash = IdStorage::make_hash(value);
        if self.ids.contains_key(&hash) {
            Some(hash)
        } else {
            None
        }
    }

    fn get_by_id(&self, id: Id) -> Option<&String> {
        self.ids.get(&id)
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn new() -> IdStorage {
        IdStorage {
            ids: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_empty() {
        let storage = IdStorage::new();
        assert_eq!(storage.get_by_id(100), None);
    }

    #[test]
    fn test_insert_one() {
        let mut storage = IdStorage::new();
        let input = "123456789";
        let id = storage.add(&input);
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get_by_id(id).unwrap(), input);
    }

    #[test]
    fn test_insert_repeat() {
        let mut storage = IdStorage::new();
        let input = "123456789";
        for _ in 0..100 {
            storage.add(&input);
        }
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get_by_id(100), None);
    }
}
