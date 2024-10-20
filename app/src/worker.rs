use std::sync::mpsc::Receiver;

pub struct Worker {
    receiver: Receiver<String>,
    id: usize,
    name: String,
}

impl Worker {
    pub fn new(receiver: Receiver<String>, id: usize, name: String) -> Worker {
        Worker { receiver, id, name }
    }

    pub fn run(&self) {
        loop {
            match self.receiver.recv() {
                Ok(_) => {
                    // Simulate some work
                }
                Err(err) => {
                    println!(
                        "Thread {}-{} has stopped, due to error: {err}",
                        self.id, self.name
                    );
                    break;
                }
            }
        }
    }
}
