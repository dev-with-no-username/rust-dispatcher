use std::{error, sync::mpsc::Sender};

pub struct Dispatcher {
    senders: Vec<Sender<JobType>>,
}

#[allow(warnings)]
impl Dispatcher {
    pub fn new(senders: Vec<Sender<JobType>>) -> Self {
        Dispatcher { senders }
    }

    pub fn dispatch(&self, job: Job, index: usize) {
        // use index to distribute workload evenly
        match self.senders[index].send(JobType::Data(job)) {
            Ok(_) => {}
            Err(err) => {
                println!("dispatch error: {err}")
            }
        }
    }

    pub fn graceful_shutdown(&self) {
        for tx in &self.senders {
            tx.send(JobType::None);
        }
    }
}

pub enum JobType {
    Data(Job),
    None,
}

pub struct Job {
    pub name: String,
    pub function: Box<dyn Fn() -> Result<String, Box<dyn error::Error>> + Send>,
}

impl Job {
    pub fn new(
        name: String,
        function: impl Fn() -> Result<String, Box<dyn error::Error>> + Send + 'static,
    ) -> Self {
        Job {
            name,
            function: Box::new(function),
        }
    }
}
