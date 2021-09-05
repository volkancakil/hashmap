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

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > self.buckets.len() * 2 / 4 {
            self.resize();
        }
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let bucket = hasher.finish() as usize % self.buckets.len();
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

    pub fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => n * 2,
        };

        let mut new_buckets = vec![Bucket::new(); target_size];

        for Bucket { items } in self.buckets.drain(..) {
            items.iter().for_each(|(key, value)| {
                let key = key.clone();
                let value = value.clone();
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                let bucket = hasher.finish() as usize % new_buckets.len();
                new_buckets[bucket].items.push((key, value));
            });
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
        map.insert("foo", 42);
    }
}
