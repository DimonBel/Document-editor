use mongodb::{Client, Database};
use crate::error::MongoError;
#[derive(Clone)]
pub struct MongoDb { pub client: Client, pub db_name: String }
impl MongoDb {
    pub async fn connect(url: &str, db_name: impl Into<String>) -> Result<Self, MongoError> {
        let client = Client::with_uri_str(url).await?;
        Ok(Self { client, db_name: db_name.into() })
    }
    pub fn database(&self) -> Database { self.client.database(&self.db_name) }
}
