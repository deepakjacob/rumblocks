use std::any::Any;
use std::collections::HashMap;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};

enum TaskType {
    CpuLoadAverageTask,
    PrintTimeTask,
    MemInfoTask(HashMap<String, Box<dyn Any + Send + Sync>>),
}

struct Task {
    name: String,
    execution_interval_secs: u32,
    task_type: TaskType,
}

impl Task {
    fn new(name: String, execution_interval_secs: u32, task_type: TaskType) -> Self {
        Self {
            name,
            execution_interval_secs,
            task_type,
        }
    }
}

async fn task_executor() -> Result<(Sender<Task>, Receiver<String>), Box<dyn std::error::Error>> {
    let (task_tx, mut task_rx) = channel::<Task>(32);
    let (status_tx, status_rx) = channel::<String>(32);

    tokio::spawn(async move {
        while let Some(task) = task_rx.recv().await {
            let interval = Duration::from_secs(task.execution_interval_secs as u64);
            let task_type = task.task_type;
            let status_tx = status_tx.clone();
            tokio::spawn(async move {
                loop {
                    match task_type {
                        TaskType::CpuLoadAverageTask => {
                            status_tx
                                .send("Running CPU Load Average Task".to_string())
                                .await
                                .unwrap();
                        }
                        TaskType::PrintTimeTask => {
                            status_tx
                                .send("Running Print Time Task".to_string())
                                .await
                                .unwrap();
                        }
                        TaskType::MemInfoTask(ref map) => {
                            status_tx
                                .send(format!("Running Mem Info Task with data: {:?}", map))
                                .await
                                .unwrap();
                        }
                    }
                    sleep(interval).await;
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

    let cpu_task = Task::new(
        "CPU Load Average".to_string(),
        5,
        TaskType::CpuLoadAverageTask,
    );
    let time_task = Task::new("Print Time".to_string(), 1, TaskType::PrintTimeTask);
    let mem_info_task = Task::new(
        "Memory Info".to_string(),
        3,
        TaskType::MemInfoTask(HashMap::new()),
    );

    task_sender.send(cpu_task).await.unwrap();
    task_sender.send(time_task).await.unwrap();
    task_sender.send(mem_info_task).await.unwrap();

    while let Some(status) = status_receiver.recv().await {
        println!("Received status: {}", status);
    }
}
