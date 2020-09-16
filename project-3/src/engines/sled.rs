use crate::Engine as KvsEngine;
use crate::Error as KvsError;
use crate::Result;
use sled::{Db, Tree};

// #[derive(Clone)]
pub struct Sled {
    db: Db,
}

impl Sled {
    pub fn new(db: Db) -> Self {
        Sled { db }
    }
}

impl KvsEngine for Sled {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let tree: &Tree = &self.db;

        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let tree: &Tree = &self.db;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let tree: &Tree = &self.db;
        tree.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
