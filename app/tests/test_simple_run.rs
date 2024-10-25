use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

use rand::distributions::{Alphanumeric, DistString};

const MAX_JOBS: i32 = 1000000;
const MAX_WORKERS: i8 = 10;

#[test]
fn simple_run() {
    run();
}

fn run() {
    // create thread_pool for various threads and start receivers waiting for jobs
    let thread_pool = create_thread_pool(MAX_WORKERS);
    let mut num = 1;
    let mut worker_pool = vec![];
    let mut senders = vec![];

    for ch in thread_pool {
        // create a senders vec for outer dispatch operations: needs to create a shared reference (Arc)
        // so that it wont be deallocated by Rust and the receiver doesn't stop to listen to work (because if
        // the sender will be deallocated, then the receiver goes in error). The Arc doesn't need <Mutex> cause
        // the Sender<> implements Send and Sync traits that are needed to use this value inside a thread
        senders.push(Arc::new(ch.0.clone()));

        // create a vec to be able to wait all workers after
        worker_pool.push(thread::spawn(move || {
            worker(
                ch.1,
                num,
                Alphanumeric.sample_string(&mut rand::thread_rng(), 4),
            )
        }));

        num += 1;
    }

    // dispatch some jobs to receivers (workers)
    for (i, sen) in senders.iter().enumerate() {
        // iterate through Arc references and clone each one, to ensure that dispatcher() doesn't own the value,
        // otherwise it will deallocate the sender, make the receiver go in error stopping its execution
        //
        // if I had used a reference of Sender<Option<String>>, I should have created the "senders" vec with
        // only "ch.0.clone()", but then in the dispatcher() I couldn't use the thread to spawn execution,
        // due to the fact that a thread needs to own the value and doesn't accept reference (&)
        dispatcher(
            Arc::clone(sen),
            Some(Alphanumeric.sample_string(&mut rand::thread_rng(), i + 1) + "sender"),
        );
    }

    // wait for all jobs to finish
    for work in worker_pool {
        work.join().unwrap()
    }
}

fn create_thread_pool(max_workers: i8) -> Vec<(Sender<Option<String>>, Receiver<Option<String>>)> {
    let mut thread_pool = vec![];
    for _ in 0..max_workers {
        let ch = mpsc::channel();
        thread_pool.push(ch);
    }
    thread_pool
}

fn worker(receiver: Receiver<Option<String>>, id: usize, name: String) {
    loop {
        match receiver.recv() {
            Ok(Some(_message)) => {
                // Simulate some work
                // println!("thread {id}-{name} received: {:#?}", message);
            }
            Ok(None) => {
                println!("gracefully shutdown {id}-{name} worker");
                break;
            }
            Err(err) => {
                println!("worker {id}-{name} has stopped, due to error: {err}");
                break;
            }
        }
    }
}

fn dispatcher(sender: Arc<Sender<Option<String>>>, job: Option<String>) {
    thread::spawn(move || {
        for n in 0..MAX_JOBS {
            match job {
                Some(ref message) => {
                    match sender.send(format!("some text sent by {n}-{message}").into()) {
                        Ok(_) => {
                            // send None to make workers stop waiting for jobs
                            if n == MAX_JOBS - 1 {
                                let _ = sender.send(None);
                                break;
                            }
                        }
                        Err(err) => {
                            println!("job sending error: {err}")
                        }
                    }
                }
                None => match sender.send(None) {
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        println!("job gracefully shutdown goes wrong, due to error: {err}")
                    }
                },
            }
        }
    });
}
