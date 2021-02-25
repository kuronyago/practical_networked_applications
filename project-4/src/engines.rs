#[derive(Clone)]
struct Store;

impl crate::Engine for Store {
    fn set(&self, key: String, value: String) -> crate::StoreResult<()> {
        todo!()
    }

    fn get(&self, key: String) -> crate::StoreResult<Option<String>> {
        todo!()
    }

    fn remove(&self, key: String) -> crate::StoreResult<()> {
        todo!()
    }
}
