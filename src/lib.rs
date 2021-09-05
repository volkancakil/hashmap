use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

const INITIAL_NBUCKETS: usize = 1;

#[derive(Clone)]
struct Bucket<K, V> {
    items: Vec<(K, V)>,
}

impl<K, V> Bucket<K, V> {
    fn new() -> Self {
        Bucket { items: Vec::new() }
    }
}

pub struct HashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    items: usize,
}

impl<K, V> Default for HashMap<K, V>
where
    K: Hash + Eq + std::clone::Clone,
    V: std::clone::Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq + std::clone::Clone,
    V: std::clone::Clone,
{
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }

    fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.buckets.len()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > self.buckets.len() * 2 / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];

        self.items += 1;
        for (ref ekey, ref mut evalue) in bucket.items.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        bucket.items.push((key, value));
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket = self.bucket(key);
        self.buckets[bucket]
            .items
            .iter()
            .find(|&(ref ekey, _)| ekey == key)
            .map(|&(_, ref v)| v)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let removed = bucket.items.iter().position(|&(ref ekey, _)| ekey == key)?;
        self.items -= 1;
        Some(bucket.items.swap_remove(removed).1)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    pub fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => n * 2,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Bucket::new()));

        for (key, value) in self
            .buckets
            .iter_mut()
            .flat_map(|bucket| bucket.items.drain(..))
        {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = hasher.finish() as usize % new_buckets.len();
            new_buckets[bucket].items.push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("foo", 42);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.len(), 0);
        assert_eq!(map.get(&"foo"), None);
    }
}

