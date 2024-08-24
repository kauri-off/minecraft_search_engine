use std::{io::Result, net::SocketAddr, sync::Arc};

use checker::{get_motd, license};
use tokio::sync::mpsc::{self, Receiver, Sender};
use utils::{check_port_open, get_random_ip};

mod packet;
mod packets;
mod types;

mod checker;
mod utils;

async fn process_ip(ip: SocketAddr) -> Result<()> {
    println!("[/] {}", ip);

    let motd = get_motd(ip).await?;
    let licensed = license(ip, motd.protocol).await;

    println!(
        "[+] ({}) -> {} | {} | {}/{} | License: {:?}",
        ip.ip(),
        motd.version,
        motd.motd,
        motd.online,
        motd.max_online,
        licensed
    );

    Ok(())
}

async fn wait_for_ip(mut rx: Receiver<SocketAddr>) {
    while let Some(ip) = rx.recv().await {
        tokio::spawn(process_ip(ip));
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
    println!("Minectaft Search Engine --- Starting");
    let threads = 30;

    let (tx, rx) = mpsc::channel(256);
    let reciever_thread = tokio::spawn(wait_for_ip(rx));

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
