use std::{env, io::Result, net::SocketAddr, sync::Arc};

use checker::get_full_info;
use colored::Colorize;
use database::MongoDBClient;
use mongodb::bson::doc;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinSet,
    time::{sleep, timeout, Duration},
};
use utils::{check_port_open, get_random_ip};

mod packet;
mod packets;
mod types;

mod checker;
mod database;
mod utils;

async fn process_ip(ip: SocketAddr, db: Arc<Mutex<MongoDBClient>>) -> Result<()> {
    let info = get_full_info(ip).await?;

    db.lock().await.add(&info).await?;

    if info["license"].as_i64().unwrap() != 0 {
        return Ok(());
    }

    println!(
        "[+] ({}) -> {} | {}/{}",
        info["ip"].as_str().unwrap(),
        info["status"]["version"]["name"].as_str().unwrap().red(),
        info["status"]["players"]["online"].as_i64().unwrap(),
        info["status"]["players"]["max"].as_i64().unwrap(),
    );

    Ok(())
}

async fn wait_for_ip(mut rx: Receiver<SocketAddr>) {
    let db = MongoDBClient::new().await;

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

async fn update_ip(ip: SocketAddr, db: Arc<Mutex<MongoDBClient>>) -> Result<()> {
    let info = get_full_info(ip).await?;

    db.lock()
        .await
        .servers
        .update_one(
            doc! {"ip": ip.ip().to_string()},
            doc! {
            "$set": {
                "status.players.online": info["status"]["players"]["online"].as_i64().unwrap(),
                "status.players.max": info["status"]["players"]["max"].as_i64().unwrap()
            },
            "$addToSet": {
                "status.players.sample": {
                    "$each": mongodb::bson::to_bson(info["status"]["players"]["sample"].as_array().unwrap_or(&vec![])).unwrap()
                }
            }
            },
        )
        .await
        .unwrap();

    Ok(())
}

async fn update() -> Result<()> {
    let db = MongoDBClient::new().await;

    let servers = db.lock().await.get_all().await.unwrap();

    println!("Updating: {} servers", servers.len());

    for chunk in servers.chunks(100) {
        let mut set = JoinSet::new();

        for server in chunk {
            let db_clone = db.clone();
            set.spawn(timeout(
                Duration::from_secs(5),
                update_ip(
                    format!(
                        "{}:{}",
                        server["ip"].as_str().unwrap(),
                        server["port"].as_str().unwrap()
                    )
                    .parse()
                    .unwrap(),
                    db_clone,
                ),
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

async fn update_loop() {
    loop {
        update().await.unwrap();

        sleep(Duration::from_secs(5 * 60)).await;
    }
}

#[tokio::main]
async fn main() {
    colored::control::set_override(true);
    println!("Minectaft Search Engine --- {}", "Starting".green());

    let threads: i32 = env::var("THREADS")
        .unwrap_or("900".to_string())
        .parse()
        .unwrap();

    let update_thread = tokio::spawn(update_loop());

    let (tx, rx) = mpsc::channel(256);
    let reciever_thread = tokio::spawn(wait_for_ip(rx));

    let mut generators = Vec::new();
    let tx = Arc::new(tx);

    for _ in 0..threads {
        generators.push(tokio::spawn(generator(tx.clone())))
    }

    reciever_thread.await.unwrap();
    update_thread.await.unwrap();

    for generator in generators {
        generator.await.unwrap();
    }
}
