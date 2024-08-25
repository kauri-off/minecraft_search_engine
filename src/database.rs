use std::sync::Arc;

use rusqlite::{params, Connection, Result};
use tokio::sync::Mutex;

use crate::checker::ServerStatus;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Arc<Mutex<Self>>> {
        let conn = Connection::open(path)?;
        let db = Database { conn };

        db.init_table()?;

        Ok(Arc::new(Mutex::new(db)))
    }

    fn init_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS servers (
               	version	TEXT,
               	motd	TEXT,
               	online	INTEGER,
               	max_online	INTEGER,
               	license	INTEGER
            );",
            [],
        )?;
        Ok(())
    }

    pub fn add(&self, status: &ServerStatus, license: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO servers (version, motd, online, max_online, license) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                status.version,
                status.motd,
                status.online,
                status.max_online,
                license
            ],
        )?;
        Ok(())
    }
}
