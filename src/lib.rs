// #![feature(nill)]
#![allow(unused_must_use)]

use std::{
    borrow::Borrow,
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

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

pub struct OccupiedEntry<'a, K, V> {
    entry: &'a mut (K, V),
}
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K,
    map: &'a mut HashMap<K, V>,
    bucket: usize,
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + Eq,
    {
        self.map.buckets[self.bucket].items.push((self.key, value));
        self.map.items += 1;
        &mut self.map.buckets[self.bucket].items.last_mut().unwrap().1
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Entry::Occupied(element) => &mut element.entry.1,
            Entry::Vacant(element) => element.insert(value),
        }
    }
    pub fn or_insert_with<F>(self, maker: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(element) => &mut element.entry.1,
            Entry::Vacant(element) => element.insert(maker()),
        }
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(Default::default)
    }
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

    fn bucket<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.buckets.is_empty() {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        Some(hasher.finish() as usize % self.buckets.len())
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        // let bucket = self.bucket(&key);
        // let bucket = &mut self.buckets[bucket];

        // match bucket.items.iter_mut().find(|&&mut (ref ekey, _)| ekey == &key) {
        //     Some(entry) => Entry::Occupied(OccupiedEntry { entry: unsafe { mem::transmute(entry) } }),
        //     None => Entry::Vacant(VacantEntry {
        //         key,
        //         bucket,
        //     }),
        // }

        let bucket = self.bucket(&key).expect("buckets.is_empty() handled above");

        match self.buckets[bucket]
            .items
            .iter()
            .position(|&(ref ekey, _)| ekey == &key)
        {
            Some(index) => Entry::Occupied(OccupiedEntry {
                entry: &mut self.buckets[bucket].items[index],
            }),
            None => Entry::Vacant(VacantEntry {
                map: self,
                key,
                bucket,
            }),
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > self.buckets.len() * 2 / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key).expect("buckets.is_empty() handled above");
        let bucket = &mut self.buckets[bucket];
        for (ref ekey, ref mut evalue) in bucket.items.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        self.items += 1;
        bucket.items.push((key, value));
        None
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        // let bucket = self.bucket(key);
        // self.buckets[bucket]
        //     .items
        //     .iter()
        //     .any(|(ref ekey, _)| ekey == key)

        self.get(key).is_some()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        self.buckets[bucket]
            .items
            .iter()
            .find(|(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        let bucket = &mut self.buckets[bucket];
        let removed = bucket
            .items
            .iter()
            .position(|&(ref ekey, _)| ekey.borrow() == key)?;
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

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.items.get(self.at) {
                    Some(&(ref k, ref v)) => {
                        self.at += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
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
        assert!(map.contains_key(&"foo"));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.len(), 0);
        assert!(!map.contains_key(&"foo"));
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("bar", 43);
        map.insert("baz", 44);
        map.insert("quox", 45);

        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 42),
                "bar" => assert_eq!(v, 43),
                "baz" => assert_eq!(v, 44),
                "quox" => assert_eq!(v, 45),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 4);
    }
}
