use std::{
    env::args,
    net::IpAddr,
    time::{Duration, Instant},
};

use serde::Deserialize;
use tokio::{net::TcpStream, spawn, time::sleep};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
struct Resp {
    staticnodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
struct Node {
    location: String,
    ip: IpAddr,
}

async fn get_nodes() -> Result<Vec<Node>> {
    let resp = reqwest::get("https://mudfish.net/api/staticnodes")
        .await?
        .bytes()
        .await?;
    Ok(serde_json::from_slice::<Resp>(&resp)?.staticnodes)
}

#[tokio::main]
async fn main() {
    println!(
        "{:^49} : {:^15} : {:^7} : {:^7} : {:^7}",
        "Location", "IP", "MIN", "MAX", "AVG"
    );
    let filter = args().nth(1).unwrap().to_uppercase();
    for node in get_nodes().await.unwrap() {
        if node.location.contains(&filter) {
            spawn(async move {
                let mut rtt_min = f64::MAX;
                let mut rtt_max = f64::MIN;
                let mut rtt_tot = 0f64;
                let mut cnt = 0;
                for _ in 0..4 {
                    let now = Instant::now();
                    if let Ok(_) = TcpStream::connect((node.ip, 1723)).await {
                        let rtt = now.elapsed().as_secs_f64() * 1000f64;
                        rtt_min = rtt_min.min(rtt);
                        rtt_max = rtt_max.max(rtt);
                        rtt_tot = rtt_tot + rtt;
                        cnt = cnt + 1;
                    }
                }
                if cnt > 0 {
                    println!(
                        "{:49} : {:15?} : {:7.2} : {:7.2} : {:7.2}",
                        node.location,
                        node.ip,
                        rtt_min,
                        rtt_max,
                        rtt_tot / cnt as f64
                    )
                }
            });
            sleep(Duration::from_millis(100)).await
        }
    }
    sleep(Duration::from_secs(5)).await
}
