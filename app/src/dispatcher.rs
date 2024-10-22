use std::{error, sync::mpsc::Sender, thread};

pub struct Dispatcher {
    senders: Vec<Sender<String>>,
    jobs: Vec<Job>,
}

impl Dispatcher {
    pub fn new(senders: Vec<Sender<String>>, jobs: Vec<Job>) -> Dispatcher {
        Dispatcher { senders, jobs }
    }

    pub fn dispatch(self) {
        let mut index = 0;
        thread::spawn(move || {
            for n in 1..self.jobs.len() {
                // use index to distribute workload evenly
                match self.senders[index].send(format!("Some text sent by {n} sender")) {
                    Ok(_) => match self.jobs[n].run() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Job error {err}")
                        }
                    },
                    Err(err) => {
                        println!("Sender error {err}")
                    }
                }
                if index == self.senders.len() - 1 {
                    index = 0
                } else {
                    index += 1
                }
            }
        });
    }
}

pub struct Job {
    pub name: String,
    pub function: Box<dyn Fn() -> Result<String, Box<dyn error::Error>> + Send>,
}

impl Job {
    pub fn new(
        name: String,
        function: impl Fn() -> Result<String, Box<dyn error::Error>> + Send + 'static,
    ) -> Job {
        Job {
            name,
            function: Box::new(function),
        }
    }

    fn run(&self) -> Result<String, Box<dyn error::Error>> {
        (self.function)()
    }
}
