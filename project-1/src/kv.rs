use std::collections::HashMap;

/// The `KvStore` stores `String` to `String` pairs.
///
/// `KvStore` is based on the standard implementation of hashmap -
/// `std::collections::HashMap`
///
/// How To Use:
/// ```rust
/// use project_1::KvStore;
///
/// fn main() {
///     let mut storage: KvStore = KvStore::new();
///     storage.set(String::from("key"), String::from("value"));
///     let value: Option<String> = storage.get(String::from("key"));
///     if let Some(s) = value {
///         println!("{}", s.as_str());         
///     } else {
///         println!("none");
///     }
/// }
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// Set the value of a String key to a String with.
    ///
    /// If the key exists, the value will be overwritten.
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Gets the String value of a given String key.
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    /// Remove a given String key.
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
