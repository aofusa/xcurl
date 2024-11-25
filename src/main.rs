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

    #[arg(short, long, help = "繰り返しを行う時間を秒単位で指定します。指定された時間内で可能な限り繰り返し実行します。このオプションを使用するとき--repeatは無視されます")]
    time: Option<usize>,

    #[arg(short, long, default_value_t = 0, help = "各実行間の待機時間をミリ秒単位で指定。デフォルトは待機なし。")]
    wait: u64,

    #[arg(short, long, default_value_t = 1, help = "並列で実行する数を指定。0の場合可能な限り並列数を増やす。")]
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
struct Metrics {
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

fn statistics(response: &[Response]) -> Metrics {
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

    Metrics {
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

    if args.parallel > 0 {
        for _parallels in 0..args.parallel {
            let curl_args = curl_args.clone();
            let tx = tx.clone();
            handle.push(if args.time.is_some() {
                let now = Instant::now();
                let time = args.time.unwrap();
                tokio::spawn(async move {
                    while now.elapsed() < Duration::from_secs(time.try_into().unwrap()) {
                        let response = call(&curl_args).await;
                        if let Err(_) = tx.send(response).await { warn!("receiver dropped") }
                        sleep(Duration::from_millis(args.wait)).await;
                    }
                })
            } else {
                tokio::spawn(async move {
                    for _repeat in 0..args.repeat {
                        let response = call(&curl_args).await;
                        if let Err(_) = tx.send(response).await { warn!("receiver dropped") }
                        sleep(Duration::from_millis(args.wait)).await;
                    }
                })
            });
        }
    } else {
        handle.push(tokio::spawn(async move {
            if args.time.is_some() {
                let now = Instant::now();
                let time = args.time.unwrap();
                let mut inner_handle = Vec::new();
                while now.elapsed() < Duration::from_secs(time.try_into().unwrap()) {
                    let curl_args = curl_args.clone();
                    let tx = tx.clone();
                    inner_handle.push(tokio::spawn(async move {
                        let response = call(&curl_args).await;
                        if let Err(_) = tx.send(response).await { warn!("receiver dropped") }
                        sleep(Duration::from_millis(args.wait)).await;
                    }));
                }
                for x in inner_handle { x.abort() }
            } else {
                let mut inner_handle = Vec::new();
                for _repeat in 0..args.repeat {
                    let curl_args = curl_args.clone();
                    let tx = tx.clone();
                    inner_handle.push(tokio::spawn(async move {
                        let response = call(&curl_args).await;
                        if let Err(_) = tx.send(response).await { warn!("receiver dropped") }
                        sleep(Duration::from_millis(args.wait)).await;
                    }));
                }
                for x in inner_handle { x.await.unwrap() }
            }
        }));
    }

    let mut response = Vec::new();
    if args.time.is_some() {
        let now = Instant::now();
        if let Some(time) = args.time {
            let mut count = 0;
            while now.elapsed() < Duration::from_secs(time.try_into().unwrap()) {
                if let Some(msg) = rx.recv().await {
                    response.push(msg);
                    count += 1;
                    eprint!("elapsed: {:?}, count: {}, running...\r", now.elapsed(), count);
                }
            }
        }
    } else {
        let count = {
            if args.parallel > 0 {
                args.parallel*args.repeat
            } else {
                args.repeat
            }
        };
        for index in 1..(count+1) {
            if let Some(msg) = rx.recv().await {
                response.push(msg);
                eprint!("[{}/{}] running...\r", index, args.parallel*args.repeat);
            }
        }
    }
    debug!("{:?}", response);

    let metrics = statistics(&response);
    debug!("{:?}", metrics);
    println!("{}", serde_json::to_string(&metrics)?);

    for x in handle { x.abort() }
    Ok(())
}
