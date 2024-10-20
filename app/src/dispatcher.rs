use std::{sync::mpsc::Sender, thread};

use crate::MAX_JOBS_ROUND_ROBIN;

pub struct Dispatcher {
    senders: Vec<Sender<String>>,
}

impl Dispatcher {
    pub fn new(senders: Vec<Sender<String>>) -> Dispatcher {
        Dispatcher { senders }
    }

    pub fn dispatch(self) {
        let mut index = 0;
        thread::spawn(move || {
            for n in 1..MAX_JOBS_ROUND_ROBIN {
                // use index to distribute workload evenly
                match self.senders[index].send(format!("Some text sent by {n} sender")) {
                    Ok(_) => {
                        // println!("Job sent to a worker idle by {n}-{name} sender");
                    }
                    Err(err) => {
                        println!("Error {err}")
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
