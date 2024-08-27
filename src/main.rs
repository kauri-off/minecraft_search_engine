use std::{env, io::Result, net::SocketAddr, sync::Arc};

use checker::{get_full_info, get_motd, license};
use colored::Colorize;
use database::Database;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use utils::{check_port_open, get_random_ip};

mod packet;
mod packets;
mod types;

mod checker;
mod database;
mod utils;

async fn process_ip(ip: SocketAddr, db: Arc<Mutex<Database>>) -> Result<()> {
    let info = get_full_info(ip).await?;

    let (prefix, license) = match info.license {
        1 => ("/", "License: Yes".to_string()),
        0 => ("+", "License: No".green().to_string()),
        -1 => ("-", "License: Error".to_string()),
        _ => ("", "".to_string()),
    };

    println!(
        "[{}] ({}) -> {} | {} | {}/{} | {}",
        prefix,
        info.ip,
        info.version.red(),
        info.description.blue(),
        info.online,
        info.max_online,
        license
    );

    db.lock().await.add(&info).unwrap();

    Ok(())
}

async fn wait_for_ip(mut rx: Receiver<SocketAddr>, path: String) {
    let db = Database::new(&path).unwrap();

    while let Some(ip) = rx.recv().await {
        tokio::spawn(process_ip(ip, db.clone()));
    }
}

async fn generator(tx: Arc<Sender<SocketAddr>>) {
    loop {
        let ip = get_random_ip();

        if check_port_open(ip, 25565).await {
            if let Err(_) = tx.send(SocketAddr::new(ip, 25565)).await {
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    colored::control::set_override(true);
    println!("Minectaft Search Engine --- {}", "Starting".green());

    let threads: i32 = env::var("THREADS")
        .unwrap_or("1000".to_string())
        .parse()
        .unwrap();
    let path = env::var("DB").unwrap_or("/app/data/database.db".to_string());

    let (tx, rx) = mpsc::channel(256);
    let reciever_thread = tokio::spawn(wait_for_ip(rx, path));

    let mut generators = Vec::new();
    let tx = Arc::new(tx);

    for _ in 0..threads {
        generators.push(tokio::spawn(generator(tx.clone())))
    }

    reciever_thread.await.unwrap();

    for generator in generators {
        generator.await.unwrap();
    }
}
