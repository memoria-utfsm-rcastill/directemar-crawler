extern crate mongodb;

use self::mongodb::{Client, ThreadedClient};
use self::mongodb::db::ThreadedDatabase;
use self::mongodb::coll::results::InsertOneResult;
use self::mongodb::error::Error as MongoDBErr;
use bson::Document;

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {
    pub fn with_connection_string(connection_string: &str) -> Result<MongoConnection, MongoDBErr> {
        Ok(MongoConnection {
            client: Client::with_uri(connection_string)?,
        })
    }

    pub fn insert<T: Into<Document>>(
        &self,
        db: &str,
        coll: &str,
        doc: T,
    ) -> mongodb::Result<InsertOneResult> {
        self.client.db(db).collection(coll).insert_one(
            doc.into(),
            None,
        )
    }
}