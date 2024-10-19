use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread};

use rand::distributions::{Alphanumeric, DistString};

const MAX_JOBS: i32 = 20; // 2000000 -> 1.7967563 seconds, 10000000 -> 9.310581 seconds
const MAX_WORKERS: i8 = 10;

pub fn run() {
    // create channels for various threads and start receivers waiting for jobs
    let channels = create_channels();
    let mut num = 1;
    let mut handles = vec![];
    let mut senders = vec![];
    for ch in channels {
        // create a senders vec for outer dispatch operations: needs to create a shared reference (Arc<Mutex>)
        // so that it wont be deallocated by Rust and the receiver doesn't stop to listen to work (because if
        // the sender will be deallocated, then the receiver goes in error)
        senders.push(Arc::new(Mutex::new(ch.0.clone())));

        // create a vec to be able to wait all workers after
        handles.push(thread::spawn(move || 
            worker(ch.1, num, Alphanumeric.sample_string(&mut rand::thread_rng(), 4))
        ));

        num += 1;
    }

    // dispatch some jobs to receivers
    for (i, sen) in senders.iter().enumerate() {
        // iterate through Arc references and clone each one, to ensure that dispatcher() doesn't own the value, otherwise
        // it will deallocate the sender, make the receiver go in error stopping its execution
        // 
        // if I had used a reference of Sender<String>, I should have created the "senders" vec with only "ch.0.clone()",
        // but then in the dispatcher() I couldn't use the thread to spawn execution, due to the fact that a thread
        // needs to own the value and doesn't accept reference (&)
        dispatcher(Arc::clone(sen), Alphanumeric.sample_string(&mut rand::thread_rng(), i+1));
    }

    // wait for all jobs to finish
    for hand in handles {
        hand.join().unwrap()
    }
}

fn worker(receiver: Receiver<String>, id: usize, name: String) {
    loop {
        match receiver.recv() {
            Ok(message) => {
                // Simulate some work
                println!("Thread {id}-{name} received: {message}");
            }
            Err(err) => {
                println!("Thread {id}-{name} has stopped, due to error: {err}");
                break;
            }
        }
    }
}

fn dispatcher(sender: Arc<Mutex<Sender<String>>>, name: String) {
    thread::spawn(move || {
        for n in 1..MAX_JOBS {
            match sender.lock().unwrap().send(format!("Some text sent by {n}-{name} sender")) {
                Ok(_) => {
                    println!("Job sent to a worker idle by {n}-{name} sender"); 
                },
                Err(err) => {
                    println!("Error {err}")
                },
            }
        }
    });
}

fn create_channels() -> Vec<(Sender<String>, Receiver<String>)> {
    let mut channels = vec![];
    for _ in 1..MAX_WORKERS {
        let ch = mpsc::channel();
        channels.push(ch);
    }
    channels
}
