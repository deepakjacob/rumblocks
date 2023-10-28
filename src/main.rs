mod format;

use std::process::Command;

use chrono::Local;
use format::{
    cpu_load_avg_format, date_info_format, mem_format, network_io_format, wifi_info_format,
};
use std::thread::sleep as sleepstd;
use systemstat::{Platform, System};
use tokio::runtime::Builder;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};

const NET_EXECUTABLE: &str = "iw";
const NET_INTERFACE_TO_LISTEN: &str = "wlo1";

const GB: f32 = 1024.0 * 1024.0 * 1024.0;

enum TaskType {
    CpuLoadAverage(u32, fn() -> String),
    DateInfo(u32, fn() -> String),
    MemInfo(u32, fn() -> String),
    WifiInfo(u32, fn() -> String),
    NetworkIo(u32, fn() -> String),
}

async fn task_executor() -> Result<(Sender<TaskType>, Receiver<String>), Box<dyn std::error::Error>>
{
    let (task_tx, mut task_rx) = channel::<TaskType>(32);
    let (status_tx, status_rx) = channel::<String>(32);

    tokio::spawn(async move {
        while let Some(task_type) = task_rx.recv().await {
            let status_tx = status_tx.clone();
            tokio::spawn(async move {
                loop {
                    match task_type {
                        TaskType::CpuLoadAverage(interval, execute_fn)
                        | TaskType::DateInfo(interval, execute_fn)
                        | TaskType::MemInfo(interval, execute_fn)
                        | TaskType::WifiInfo(interval, execute_fn)
                        | TaskType::NetworkIo(interval, execute_fn) => {
                            status_tx.send(execute_fn()).await.unwrap();
                            sleep(Duration::from_secs(interval as u64)).await;
                        }
                    }
                }
            });
        }
    });

    Ok((task_tx, status_rx))
}

#[tokio::main]
async fn main() {
    let _runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    let (task_sender, mut status_receiver) = task_executor().await.unwrap();

    let cpu_load_avg = TaskType::CpuLoadAverage(3, || {
        let value = if let Ok(loadavg) = System::new().load_average() {
            loadavg.one
        } else {
            0.0
        };
        cpu_load_avg_format(value)
    });

    let date_info = TaskType::DateInfo(30, || date_info_format(Local::now()));

    let mem_info = TaskType::MemInfo(3, || {
        let value = if let Ok(mem) = System::new().memory() {
            let total_ram = (mem.total.0 as f32) / GB;
            let used_ram = ((mem.total.0 - mem.free.0) as f32) / GB;
            (used_ram, total_ram)
        } else {
            (0.0, 0.0)
        };

        mem_format(value)
    });

    let wifi_info = TaskType::WifiInfo(5, || {
        let output = Command::new(NET_EXECUTABLE)
            .arg("dev")
            .arg(NET_INTERFACE_TO_LISTEN)
            .arg("link")
            .output()
            .expect("Failed to execute command");

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.contains("signal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Ok(strength) = parts[1].parse::<i32>() {
                    return wifi_info_format(strength);
                }
            }
        }
        wifi_info_format(0)
    });

    let network_io = TaskType::NetworkIo(3, || {
        let call_interval = Duration::from_secs(1); // 1 second interval
        let (rx_initial, tx_initial) = fetch_network_stats();
        sleepstd(call_interval);
        let (rx_final, tx_final) = fetch_network_stats();

        // Calculate the speed per second
        let rx_speed_per_sec =
            ((rx_final - rx_initial) as f64 / (1024.0 * 1024.0)) / call_interval.as_secs_f64();
        let tx_speed_per_sec =
            ((tx_final - tx_initial) as f64 / (1024.0 * 1024.0)) / call_interval.as_secs_f64();

        network_io_format(rx_speed_per_sec, tx_speed_per_sec)
    });

    task_sender.send(cpu_load_avg).await.unwrap();
    task_sender.send(date_info).await.unwrap();
    task_sender.send(mem_info).await.unwrap();
    task_sender.send(wifi_info).await.unwrap();
    task_sender.send(network_io).await.unwrap();

    while let Some(status) = status_receiver.recv().await {
        println!("Received status: {}", status);
    }
}
fn fetch_network_stats() -> (u64, u64) {
    let sys = System::new();
    match sys.networks() {
        Ok(netifs) => {
            if let Some(netif) = netifs.get(NET_INTERFACE_TO_LISTEN) {
                let net_stats = sys.network_stats(&netif.name).unwrap();
                return (net_stats.rx_bytes.0, net_stats.tx_bytes.0);
            } else {
                return (0, 0);
            }
        }
        _ => (0, 0),
    }
}
