//! Implementation of LFU protocol (not LRU) in Rust
//! Main focus is on performance and rustic approach rather than <insert language> in Rust.
//!
//! Implementation of http://dhruvbird.com/lfu.pdf
//!
//!
//!
//!

use bytes::Bytes;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default)]
struct FrequencyNode {
    items: Vec<String>,
}

impl FrequencyNode {
    pub fn new() -> Self {
        FrequencyNode { items: vec![] }
    }
}

/// original paper uses LFU Item but since this is private I see no reason for prefixing
#[derive(Debug, Default)]
struct Item {
    data: Bytes,
    parent: usize,
}

impl Item {
    pub fn new(data: Bytes) -> Self {
        Item { data, parent: 0 }
    }
}

#[derive(Debug, Default)]
pub struct LFU {
    // main data storage, every cache can be usually thought of as a fixed size hashmap with extra method to evict certain keys when new value is added
    items: HashMap<String, Item>,
    // list of frequency nodes mapping frequency expressed as number to a FrequencyNode
    // which is a store of keys, this may eventually be better expressed as hashmap too,
    // for the time being I'm letting this live as Vec where at each index we have (or add if needed)
    // a FrequencyNode instance
    frequency_list: Vec<FrequencyNode>,
    // instead of pointer to end we keep index of last valued elem
    tail_index: usize,
    // each cache has max allowed size for data, this does not include overhead coming
    // from implementation itself
    max_size: usize,
    // this keeps track of size of heap stored Items data
    current_size: usize,
    // useful extension of vect based LFU with history option
    history: VecDeque<String>,
}

impl LFU {
    pub fn new() -> Self {
        let frequency_head = FrequencyNode::new();
        LFU {
            items: HashMap::new(),
            max_size: 64,
            current_size: 0,
            tail_index: 0,
            frequency_list: vec![frequency_head],
            history: VecDeque::with_capacity(64),
        }
    }
    ///
    /// Builder for max_size, only outside-configurable value for cache
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// let lfu = LFU::new().max_size(1024);
    /// ```
    ///
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }
    ///
    /// Check if we have value for this key
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new().max_size(1024);
    /// assert_eq!(lfu.contains_key("a"), false);
    /// lfu.insert("a".to_string(), Bytes::from("a"));
    /// assert_eq!(lfu.contains_key("a"), true);
    /// ```
    ///
    pub fn contains_key(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }

    ///
    /// Check how many items there currently is in cache
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new();
    /// assert_eq!(lfu.current_size(), 0);
    /// lfu.insert("a".to_string(), Bytes::from("b"));
    /// assert_eq!(lfu.current_size(), 1);
    /// ```
    ///
    pub fn current_size(&self) -> usize {
        self.current_size
    }
    ///
    /// Allows to check frequency for a key of given value
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new();
    /// lfu.get_frequency("a");
    /// lfu.insert("a".to_string(), Bytes::from("b"));
    /// assert_eq!(lfu.get_frequency("a"), 0);
    /// lfu.get("a");
    /// assert_eq!(lfu.get_frequency("a"), 1);
    /// lfu.get("a");
    /// assert_eq!(lfu.get_frequency("a"), 2);
    /// lfu.get("a");
    /// assert_eq!(lfu.get_frequency("a"), 3);
    /// ```
    pub fn get_frequency(&mut self, key: &str) -> usize {
        match self.items.get(key) {
            Some(item) => item.parent,
            _ => 0,
        }
    }

    ///
    /// Get a Some(value) or None for a given key
    ///
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new();
    /// assert_eq!(lfu.get("a"), None);
    /// lfu.insert("a".to_string(), Bytes::from("b"));
    /// assert_eq!(lfu.get("a"), Some(&Bytes::from("b")));
    /// ```
    pub fn get(&mut self, key: &str) -> Option<&Bytes> {
        if let Some(item) = self.items.get_mut(key) {
            if let Some(frequency_node) = self.frequency_list.get_mut(item.parent) {
                frequency_node.items.retain(|x| x != key);
            }
            item.parent += 1;
            match self.frequency_list.get_mut(item.parent) {
                Some(frequency_node) => {
                    // we have the next fnode
                    frequency_node.items.push(key.to_owned());
                }
                None => {
                    // we need to add a node
                    let mut frequency_node = FrequencyNode::new();
                    frequency_node.items.push(key.to_owned());
                    self.frequency_list.push(frequency_node);
                }
            }
            Some(&item.data)
        } else {
            None
        }
    }
    ///
    /// Record evicted key in history
    ///
    fn add_to_history(&mut self, dropped_key: String) {
        while self.history.len() > self.max_size {
            self.history.pop_back();
        }
        self.history.push_front(dropped_key);
    }
    ///
    /// Check if key was recently dropped from cache. History has same size as max_size (remembers max_size keys)
    ///
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new().max_size(3);
    /// lfu.insert("a".to_string(), Bytes::from("42"));
    /// lfu.insert("b".to_string(), Bytes::from("43"));
    /// lfu.insert("c".to_string(), Bytes::from("43"));
    /// assert_eq!(lfu.has_evicted_recently("a"), true);
    /// ```

    pub fn has_evicted_recently(&self, key: &str) -> bool {
        self.history
            .iter()
            .any(|historical_key| historical_key.eq(key))
    }

    ///
    /// Insert a value into LFU
    ///
    ///
    /// ```
    /// use lfu_vecs::LFU;
    /// use bytes::Bytes;
    /// let mut lfu = LFU::new();
    /// lfu.insert("a".to_string(), Bytes::from("b"));
    /// lfu.insert("a".to_string(), Bytes::from("z"));
    /// assert_eq!(lfu.get("a"), Some(&Bytes::from("z")));
    /// ```
    pub fn insert(&mut self, key: String, value: Bytes) -> Option<Bytes> {
        let mut fnode_index = 0 as usize;
        while self.current_size + value.len() >= self.max_size {
            if let Some(frequency_node) = self.frequency_list.get_mut(fnode_index) {
                if let Some(key) = frequency_node.items.pop() {
                    if let Some(item) = self.items.remove(&key) {
                        self.current_size -= item.data.len();
                        self.add_to_history(key);
                    }
                };
            }
            if fnode_index == self.frequency_list.len() {
                break;
            }
            fnode_index += 1;
        }

        self.current_size += value.len();
        let previous = match self.items.insert(key.clone(), Item::new(value)) {
            Some(previous) => Some(previous.data),
            None => None,
        };
        match self.frequency_list.get_mut(0) {
            Some(frequency_node) => frequency_node.items.push(key),
            _ => unreachable!(),
        }
        previous
    }
}

#[cfg(test)]
mod tests {

    use crate::*;
    use bytes::Bytes;

    #[test]
    fn it_works() {
        let mut lfu = LFU::new();
        lfu.insert("a".to_string(), Bytes::from("42"));
        assert_eq!(lfu.get(&"a".to_string()), Some(&Bytes::from("42")));
    }
    #[test]
    fn test_max_size() {
        let lfu = LFU::new().max_size(1000);
        assert_eq!(lfu.max_size, 1000);
    }

    #[test]
    fn test_evictions() {
        let mut lfu = LFU::new().max_size(5);
        lfu.insert("a".to_string(), Bytes::from("42"));
        lfu.insert("b".to_string(), Bytes::from("43"));
        lfu.insert("c".to_string(), Bytes::from("43"));
        lfu.insert("d".to_string(), Bytes::from("43"));
        assert_eq!(lfu.current_size(), 4);
    }

    #[test]
    fn test_frequency() {
        let mut lfu = LFU::new().max_size(3);
        lfu.insert("a".to_string(), Bytes::from("42"));
        lfu.get("a");
        lfu.get("a");
        assert_eq!(lfu.get_frequency("a"), 2);
    }
}
