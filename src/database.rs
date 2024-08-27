use std::sync::Arc;

use rusqlite::{params, Connection, Result};
use tokio::sync::Mutex;

use crate::checker::Info;

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
                ip TEXT,
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

    pub fn add(&self, info: &Info) -> Result<()> {
        self.conn.execute(
            "INSERT INTO servers (ip, version, motd, online, max_online, license) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                info.ip,
                info.version,
                info.description,
                info.online,
                info.max_online,
                info.license
            ],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Info>> {
        let mut stmt = self.conn.prepare("SELECT * FROM servers")?;

        let rows: Vec<Result<Info>> = stmt
            .query_map([], |row| {
                Ok(Info {
                    ip: row.get(0)?,
                    port: "25565".to_string(),
                    version: row.get(1)?,
                    description: row.get(2)?,
                    online: row.get(3)?,
                    max_online: row.get(4)?,
                    license: row.get(5)?,
                })
            })?
            .collect();
        let successes: Vec<Info> = rows.into_iter().filter_map(Result::ok).collect();

        Ok(successes)
    }

    pub fn drop_servers(&self) -> Result<()> {
        self.conn.execute("DROP TABLE servers", [])?;
        Ok(())
    }
}
