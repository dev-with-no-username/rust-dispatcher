use std::sync::mpsc::Receiver;

use crate::dispatcher::JobType;

pub struct Worker {
    receiver: Receiver<JobType>,
    id: usize,
    name: String,
}

impl Worker {
    pub fn new(receiver: Receiver<JobType>, id: usize, name: String) -> Self {
        Worker { receiver, id, name }
    }

    pub fn run(&self) {
        // num is for log and debug purpose
        let mut num = 0;
        loop {
            match self.receiver.recv() {
                Ok(JobType::Data(job)) => {
                    // Simulate some work
                    match (job.function)() {
                        Ok(_) => {
                            // commented to not overload terminal in case of high number of jobs
                            // println!(
                            //     "job {num} in {}-{} worker, successfully completed",
                            //     self.id, self.name
                            // );
                            num += 1;
                        }
                        Err(err) => {
                            println!(
                                "job execution went wrong in {}-{} worker, due to error: {err}",
                                self.id, self.name
                            );
                            break;
                        }
                    };
                }
                Ok(JobType::None) => {
                    println!(
                        "gracefully shutdown {}-{} worker, exiting from {num}",
                        self.id, self.name
                    );
                    break;
                }
                Err(err) => {
                    println!(
                        "worker {}-{} was stopped, due to error: {err}",
                        self.id, self.name
                    );
                    break;
                }
            }
        }
    }
}
