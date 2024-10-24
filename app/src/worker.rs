use std::sync::mpsc::Receiver;

use crate::dispatcher::JobType;

pub struct Worker {
    receiver: Receiver<JobType>,
    id: usize,
    name: String,
}

impl Worker {
    pub fn new(receiver: Receiver<JobType>, id: usize, name: String) -> Worker {
        Worker { receiver, id, name }
    }

    pub fn run(&self) {
        // let mut num = 0;
        loop {
            match self.receiver.recv() {
                Ok(JobType::Data(job)) => {
                    // Simulate some work
                    match (job.function)() {
                        Ok(_) => {
                            // println!("job {num} execution successfully completed");
                            // num += 1;
                        },
                        Err(err) => {
                            println!("job execution went wrong, due to error: {err}");
                        },
                    };
                }
                Ok(JobType::None) => {
                    println!("received shutdown signal, exiting");
                    break;
                }
                Err(err) => {
                    println!(
                        "worker {}-{} has stopped, due to error: {err}",
                        self.id, self.name
                    );
                    break;
                }
            }
        }
    }
}
