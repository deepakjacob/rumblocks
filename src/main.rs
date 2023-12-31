mod format;

use std::collections::HashMap;
use std::process::Command;

use chrono::Local;
use format::{
    cpu_load_avg_format, date_info_format, mem_format, network_io_format, wifi_info_format,
    SEPARATOR,
};
use std::thread::sleep as sleepstd;
use systemstat::{Platform, System};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};

/* a linux utility to get the wifi strength e.g, iw, iwctl, wavemon etc*/
const NET_EXECUTABLE: &str = "iw";

/* the n/w interface to listen, use "ip a" to find out yours */
const NET_INTERFACE_TO_LISTEN: &str = "wlo1";

const GB: f32 = 1024.0 * 1024.0 * 1024.0;
const MB: f64 = 1024.0 * 1024.0;

/* supported blocks */
const MEM_INFO: &str = "MemInfo";
const CPU_LOAD_AVG: &str = "CpuLoadAvg";
const WIFI_INFO: &str = "WifiInfo";
const DATE_INFO: &str = "DateInfo";
const NET_IO: &str = "NetworkIo";

enum BlockType {
    /* block  -   name interval fn to be executed */
    CpuLoadAverage(String, u32, fn() -> String),
    DateInfo(String, u32, fn() -> String),
    MemInfo(String, u32, fn() -> String),
    WifiInfo(String, u32, fn() -> String),
    NetworkIo(String, u32, fn() -> String),
}

async fn block_executor(
) -> Result<(Sender<BlockType>, Receiver<(String, String)>), Box<dyn std::error::Error>> {
    let (block_tx, mut block_rx) = channel::<BlockType>(32);
    let (status_tx, status_rx) = channel::<(String, String)>(32);

    // thread spawn on each block request
    tokio::spawn(async move {
        while let Some(block_type) = block_rx.recv().await {
            let status_tx = status_tx.clone();
            // thread spawned to continuously execute each block based on the interval provided
            tokio::spawn(async move {
                loop {
                    match block_type {
                        BlockType::CpuLoadAverage(ref name, interval, execute_fn)
                        | BlockType::DateInfo(ref name, interval, execute_fn)
                        | BlockType::MemInfo(ref name, interval, execute_fn)
                        | BlockType::WifiInfo(ref name, interval, execute_fn)
                        | BlockType::NetworkIo(ref name, interval, execute_fn) => {
                            status_tx
                                .send((name.to_owned(), execute_fn()))
                                .await
                                .unwrap();
                            sleep(Duration::from_secs(interval as u64)).await;
                        }
                    }
                }
            });
        }
    });

    Ok((block_tx, status_rx))
}

#[tokio::main]
async fn main() {
    let (block_sender, mut status_receiver) = block_executor().await.unwrap();

    /* ------------------- Cpu Load Avg block ------------------------*/
    let cpu_load_avg = BlockType::CpuLoadAverage(CPU_LOAD_AVG.to_string(), 5, || {
        let value = if let Ok(loadavg) = System::new().load_average() {
            loadavg.one
        } else {
            0.0
        };
        cpu_load_avg_format(value)
    });

    /* --------------------- Date block ----------------------------*/
    let date_info =
        BlockType::DateInfo(DATE_INFO.to_string(), 30, || date_info_format(Local::now()));

    let mem_info = BlockType::MemInfo(MEM_INFO.to_string(), 5, || {
        let value = if let Ok(mem) = System::new().memory() {
            let total_ram = (mem.total.0 as f32) / GB;
            let used_ram = ((mem.total.0 - mem.free.0) as f32) / GB;
            (used_ram, total_ram)
        } else {
            (0.0, 0.0)
        };

        mem_format(value)
    });

    /* ------------------------- Wifi Info block ---------------------*/
    let wifi_info = BlockType::WifiInfo(WIFI_INFO.to_string(), 10, || {
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

    /* --------------------------- Net Io block ------------------------*/
    let network_io = BlockType::NetworkIo(NET_IO.to_string(), 3, || {
        let call_interval = Duration::from_secs(1); // 1 second interval
        let (rx_initial, tx_initial) = fetch_network_stats();
        sleepstd(call_interval);
        let (rx_final, tx_final) = fetch_network_stats();

        // Calculate the speed per second
        let rx_speed_per_sec = ((rx_final - rx_initial) as f64 / MB) / call_interval.as_secs_f64();
        let tx_speed_per_sec = ((tx_final - tx_initial) as f64 / MB) / call_interval.as_secs_f64();

        network_io_format(rx_speed_per_sec, tx_speed_per_sec)
    });

    // send the blocks for execution
    block_sender.send(cpu_load_avg).await.unwrap();
    block_sender.send(date_info).await.unwrap();
    block_sender.send(mem_info).await.unwrap();
    block_sender.send(wifi_info).await.unwrap();
    block_sender.send(network_io).await.unwrap();

    // map to store the execution results from the block
    let mut status_map: HashMap<String, String> = HashMap::new();
    while let Some(status) = status_receiver.recv().await {
        status_map.insert(
            /* name */ status.0.clone(),
            /* result */ status.1.clone(),
        );
        let mem_info = match status_map.get(MEM_INFO) {
            Some(memo_info) => memo_info,
            _ => "",
        };

        let cpu_load_avg = match status_map.get(CPU_LOAD_AVG) {
            Some(cpu_info) => cpu_info,
            _ => "",
        };

        let date_info = match status_map.get(DATE_INFO) {
            Some(date_info) => date_info,
            _ => "",
        };
        let net_io = match status_map.get(NET_IO) {
            Some(net_info) => net_info,
            _ => "",
        };
        let wifi_info = match status_map.get(WIFI_INFO) {
            Some(wifi_info) => wifi_info,
            _ => "",
        };

        update_status(&format!(
            "{} {SEPARATOR} {} {SEPARATOR} {} {SEPARATOR} {} {SEPARATOR} {}",
            wifi_info, net_io, mem_info, cpu_load_avg, date_info
        ));
    }
}

fn update_status(status: &str) {
    let _ = Command::new("xsetroot").arg("-name").arg(status).output();
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
