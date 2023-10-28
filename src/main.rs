use std::any::Any;
use std::collections::HashMap;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};

enum TaskType {
    CpuLoadAverageTask(u32),
    PrintTimeTask(u32),
    MemInfoTask(u32, HashMap<String, Box<dyn Any + Send + Sync>>),
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
                        TaskType::CpuLoadAverageTask(duration) => {
                            status_tx
                                .send("Running CPU Load Average Task".to_string())
                                .await
                                .unwrap();
                            sleep(Duration::from_secs(duration as u64)).await;
                        }
                        TaskType::PrintTimeTask(duration) => {
                            status_tx
                                .send("Running Print Time Task".to_string())
                                .await
                                .unwrap();
                            sleep(Duration::from_secs(duration as u64)).await;
                        }
                        TaskType::MemInfoTask(duration, ref map) => {
                            status_tx
                                .send(format!("Running Mem Info Task with data: {:?}", map))
                                .await
                                .unwrap();
                            sleep(Duration::from_secs(duration as u64)).await;
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
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    let (task_sender, mut status_receiver) = task_executor().await.unwrap();

    let cpu_task = TaskType::CpuLoadAverageTask(5);
    let time_task = TaskType::PrintTimeTask(1);
    let mem_info_task = TaskType::MemInfoTask(3, HashMap::new());

    task_sender.send(cpu_task).await.unwrap();
    task_sender.send(time_task).await.unwrap();
    task_sender.send(mem_info_task).await.unwrap();

    while let Some(status) = status_receiver.recv().await {
        println!("Received status: {}", status);
    }
}
