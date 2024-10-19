use std::{sync::{mpsc::{self, Receiver, Sender}, Arc}, thread, time};

use rand::distributions::{Alphanumeric, DistString};

const MAX_JOBS: i8 = 20;
const MAX_WORKERS: i8 = 10;

const MAX_JOBS_FOR_RESULTS: i32 = 2000000; 
const MAX_WORKERS_FOR_RESULTS: i8 = 10;

pub fn run() {
    // create channels for various threads and start receivers waiting for jobs
    let channels = create_channels();
    let mut num = 1;
    let mut handles = vec![];
    let mut senders = vec![];
    for ch in channels {
        // create a senders vec for outer dispatch operations: needs to create a shared reference (Arc)
        // so that it wont be deallocated by Rust and the receiver doesn't stop to listen to work (because if
        // the sender will be deallocated, then the receiver goes in error). The Arc doesn't need <Mutex> cause
        // the Senders implement Send and Sync traits that are needed to use this value inside a thread
        senders.push(Arc::new(ch.0.clone()));

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

fn dispatcher(sender: Arc<Sender<String>>, name: String) {
    thread::spawn(move || {
        for n in 1..MAX_JOBS {
            match sender.send(format!("Some text sent by {n}-{name} sender")) {
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

// below is a way to see how fast is Rust to execute all jobs, due to the fact that senders will be
// deallocated by it after dispatcher_for_results(), eventually closing the receivers automatically

pub fn run_for_velocity() {
    let now = time::Instant::now();

    // create channels for various threads and start receivers waiting for jobs
    let channels = create_channels_for_results();
    let mut num = 1;
    let mut handles = vec![];
    let mut senders = vec![];
    for ch in channels {
        // create a senders vec for outer dispatch operations
        senders.push(ch.0.clone());

        // create a vec to be able to wait all workers after
        handles.push(thread::spawn(move || 
            worker_for_results(ch.1, num, Alphanumeric.sample_string(&mut rand::thread_rng(), 4))
        ));

        num += 1;
    }

    // dispatch some jobs to receivers
    dispatcher_for_results(senders);

    // wait for all jobs to finish
    for hand in handles {
        hand.join().unwrap()
    }

    println!("\nDuration of run_for_velocity: {}\n", now.elapsed().as_secs_f32());
}

fn worker_for_results(receiver: Receiver<String>, id: usize, name: String) {
    loop {
        match receiver.recv() {
            Ok(_) => {
                // Simulate some work
            }
            Err(err) => {
                println!("Thread {id}-{name} has stopped, due to error: {err}");
                break;
            }
        }
    }
}

fn dispatcher_for_results(senders: Vec<Sender<String>>) {
    let mut index = 0;
    thread::spawn(move || {
        for n in 1..MAX_JOBS_FOR_RESULTS {
            // use index to distribute workload evenly
            match senders[index].send(format!("Some text sent by {n} sender")) {
                Ok(_) => {
                    // println!("Job sent to a worker idle by {n}-{name} sender"); 
                },
                Err(err) => {
                    println!("Error {err}")
                },
            }
            if index == senders.len() - 1 {
                index = 0
            } else {
                index += 1
            }
        }
    });
}

fn create_channels_for_results() -> Vec<(Sender<String>, Receiver<String>)> {
    let mut channels = vec![];
    for _ in 1..MAX_WORKERS_FOR_RESULTS {
        let ch = mpsc::channel();
        channels.push(ch);
    }
    channels
}