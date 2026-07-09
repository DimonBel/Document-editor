use mongodb::Collection;
use serde::{de::DeserializeOwned, Serialize};
use bson::doc;
use crate::error::MongoError;
use crate::mongo_db::MongoDb;
pub struct MongoRepo<T: Serialize + DeserializeOwned + Send + Sync + 'static> {
    pub db: MongoDb, _phantom: std::marker::PhantomData<T>,
}
impl<T: Serialize + DeserializeOwned + Send + Sync + crate::conventions::CollectionName + 'static> MongoRepo<T> {
    pub fn new(db: MongoDb) -> Self { Self { db, _phantom: std::marker::PhantomData } }
    pub fn collection(&self) -> Collection<T> { self.db.database().collection::<T>(T::COLLECTION) }
    pub async fn find_one(&self, id: &str) -> Result<Option<T>, MongoError> {
        Ok(self.collection().find_one(doc! { "_id": id }).await?)
    }
    pub async fn insert(&self, doc: &T) -> Result<(), MongoError> { self.collection().insert_one(doc).await?; Ok(()) }
    pub async fn replace(&self, id: &str, doc: &T) -> Result<(), MongoError> { self.collection().replace_one(doc! { "_id": id }, doc).await?; Ok(()) }
    pub async fn soft_delete(&self, id: &str) -> Result<(), MongoError> {
        let now = bson::DateTime::now();
        self.collection().update_one(doc! { "_id": id }, doc! { "$set": { "is_deleted": true, "deleted_at": now, "updated_at": now } }).await?;
        Ok(())
    }
}
