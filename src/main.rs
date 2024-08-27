use std::{env, io::Result, net::SocketAddr, sync::Arc};

use checker::get_full_info;
use colored::Colorize;
use database::Database;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinSet,
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

async fn update(path: &str) -> Result<()> {
    let db = Database::new(path).unwrap();

    let servers = db.lock().await.get_all().unwrap();
    db.lock().await.drop_servers().unwrap();

    println!(
        "{} Servers table is dropped for updating",
        "WARNING".yellow()
    );
    println!("{} STOP app", "DO NOT".red());
    println!("Updating: {} servers", servers.len());

    for chunk in servers.chunks(30) {
        // Создаем задачи для каждого элемента в чанке
        let mut set = JoinSet::new();

        for server in chunk {
            let db_clone = db.clone();
            set.spawn(process_ip(
                format!("{}:{}", server.ip, server.port).parse().unwrap(),
                db_clone,
            ));
        }

        while let Some(result) = set.join_next().await {
            match result {
                Ok(_) => {}
                Err(e) => eprintln!("Task failed: {:?}", e),
            }
        }
    }

    println!("Updating: {}", "done".green());

    Ok(())
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
    let update_db: bool = env::var("UPDATE")
        .unwrap_or("0".to_string())
        .parse::<u8>()
        .map(|v| v != 0)
        .unwrap_or(false);

    if update_db {
        if let Err(e) = update(&path).await {
            println!("Updating: {}, {}", "error".red(), e);
        }
    }

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
