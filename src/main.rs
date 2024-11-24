use std::cmp::{max, min};
use std::collections::HashMap;
use std::ops::{Add, Div};
use std::process::Stdio;
use std::time::Duration;
use clap::Parser;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};
use log::{debug, warn};
use serde_derive::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1, help = "curlを呼び出す回数を指定。")]
    repeat: usize,

    #[arg(short, long, default_value_t = 0, help = "各実行間の待機時間をミリ秒単位で指定。デフォルトは待機なし。")]
    wait: u64,

    #[arg(short, long, default_value_t = 1, help = "並列で実行する数を指定。0を指定した場合repeatで指定した数を上限に可能な限り並列数を増やす。")]
    parallel: usize,

    #[arg(last = true, help = "cURL引数")]
    curl_args: Vec<String>,
}

#[derive(Debug)]
struct Response {
    time: Duration,
    status_code: String,
    exit_status: i32,
}

#[derive(Serialize, Debug)]
struct Statistics {
    mean_time: u32,
    max_time: u32,
    min_time: u32,
    variance_time: u32,
    
    status_count: HashMap<String, usize>,
    error_count: usize,
}

async fn call(args: &[String]) -> Response {
    debug!("{:?}", args);

    let now = Instant::now();

    let output = Command::new("curl")
      .args(args)
      .stdout(Stdio::piped())
      .output()
      .await
      .unwrap();

    let delta = now.elapsed();

    debug!("{:?}", output);

    Response {
        time: delta,
        status_code: String::from_utf8_lossy(&output.stdout).parse().unwrap(),
        exit_status: output.status.code().unwrap(),
    }
}

fn statistics(response: &[Response]) -> Statistics {
    let time = response
      .iter()
      .map(|x| x.time.subsec_millis());

    let mean_time = time
      .clone()
      .reduce(|acc, x| acc.add(x))
      .unwrap()
      .div(response.len() as u32);

    let max_time = time
      .clone()
      .reduce(|a, b| max(a, b))
      .unwrap();

    let min_time = time
      .clone()
      .reduce(|a, b| min(a, b))
      .unwrap();

    let variance_time = time
      .clone()
      .map(|x| {
          // (&mean_time).sub(x).pow(2)
          (&mean_time).abs_diff(x)
      })
      .reduce(|acc, x| acc + x)
      .unwrap() / response.len() as u32;

    let status_count = response
      .iter()
      .map(|x| x.status_code.clone())
      .fold(HashMap::new(), |mut acc, status_code| {
          if acc.contains_key(&status_code) {
              acc.insert(status_code.clone(), acc[&status_code] + 1);
          } else {
              acc.insert(status_code.clone(), 1);
          }
          acc
      });
    
    let error_count = response
      .iter()
      .map(|x| x.exit_status.clone())
      .filter(|exit_status| *exit_status != 0)
      .collect::<Vec<i32>>()
      .len();
    
    Statistics {
        mean_time,
        max_time,
        min_time,
        variance_time,
        
        status_count,
        error_count,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    debug!("{:?}", args);

    let parallel = {
        if args.parallel > 0 {
            args.parallel
        } else {
            args.repeat
        }
    };

    let mut curl_args = args.curl_args.clone();
    if !args.curl_args.contains(&String::from("-s")) {
        curl_args.push("-s".to_string());
    }
    if !args.curl_args.contains(&String::from("-o")) {
        curl_args.push("-o".to_string());
        curl_args.push("/dev/null".to_string());
    }
    if !args.curl_args.contains(&String::from("-w")) {
        curl_args.push("-w".to_string());
        curl_args.push("%{http_code}".to_string());
    }

    let mut handle = Vec::new();

    let (tx, mut rx) = mpsc::channel(1024);

    for _parallels in 0..parallel {
        let curl_args = curl_args.clone();
        let tx = tx.clone();
        handle.push(
            tokio::spawn(async move {
                for _repeat in 0..args.repeat {
                    let response = call(&curl_args).await;
                    if let Err(_) = tx.send(response).await { warn!("receiver dropped") }
                    sleep(Duration::from_millis(args.wait)).await;
                }
            })
        );
    }

    let mut response = Vec::new();
    for index in 1..parallel*args.repeat+1 {
        if let Some(msg) = rx.recv().await {
            response.push(msg);
            eprint!("[{}/{}] running...\r", index, parallel*args.repeat);
        }
    }
    debug!("{:?}", response);

    let metrics = statistics(&response);
    debug!("{:?}", metrics);
    println!("{}", serde_json::to_string(&metrics)?);

    for x in handle { x.await? }
    Ok(())
}
