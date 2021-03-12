use super::{logger, StoreResult, ThreadPool};

use crossbeam::channel::{Receiver, Sender};
use slog::{error, o, Logger};

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct SharedQueue {
    tx: Sender<Task>,
}

impl ThreadPool for SharedQueue {
    fn new(threads: u32) -> StoreResult<Self>
    where
        Self: Sized,
    {
        let root = logger("pool");

        let (tx, rx) = crossbeam::channel::unbounded::<Task>();

        for _ in 0..threads {
            let logger = root.new(o!("key" => "value"));

            let receiver = TaskReceiver {
                rx: rx.clone(),
                logger,
            };

            std::thread::Builder::new().spawn(move || run_tasks(receiver))?;
        }
        Ok(SharedQueue { tx: tx })
    }

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        todo!()
    }
}

#[derive(Clone)]
struct TaskReceiver {
    rx: Receiver<Task>,
    logger: Logger,
}

impl Drop for TaskReceiver {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let rx = self.clone();

            if let Err(err) = std::thread::Builder::new().spawn(move || run_tasks(rx)) {
                error!(&self.logger, "failed to spawn a thread: {}", err);
            }
        }
    }
}

fn run_tasks(receiver: TaskReceiver) {
    loop {
        match receiver.rx.recv() {
            Ok(task) => {
                task();
            }
            Err(err) => {
                error!(&receiver.logger, "receive task: {}", err);
            }
        }
    }
}

pub struct ThreadSpawner;

impl ThreadPool for ThreadSpawner {
    fn new(_threads: u32) -> StoreResult<Self> {
        Ok(ThreadSpawner)
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(f);
    }
}
