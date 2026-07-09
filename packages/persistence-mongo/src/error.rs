use thiserror::Error;
#[derive(Debug, Error)]
pub enum MongoError {
    #[error("mongodb: {0}")] Mongo(#[from] mongodb::error::Error),
    #[error("bson: {0}")] Bson(#[from] bson::ser::Error),
    #[error("not found")] NotFound,
}
