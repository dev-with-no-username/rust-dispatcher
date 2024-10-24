use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle}, time,
};

use dispatcher::{Dispatcher, Job, JobType};
use rand::distributions::{Alphanumeric, DistString};
use worker::Worker;

pub mod dispatcher;
pub mod worker;

const MAX_JOBS: i8 = 100;
const MAX_WORKERS: i8 = 10;

const MAX_JOBS_ROUND_ROBIN: i32 = 2000000;
const MAX_WORKERS_ROUND_ROBIN: i8 = 20;

pub fn run() {
    // create thread_pool for various threads and start receivers waiting for jobs
    let thread_pool = create_thread_pool(MAX_WORKERS);
    let mut num = 1;
    let mut worker_pool = vec![];
    let mut senders = vec![];

    for ch in thread_pool {
        // create a senders vec for outer dispatch operations: needs to create a shared reference (Arc)
        // so that it wont be deallocated by Rust and the receiver doesn't stop to listen to work (because if
        // the sender will be deallocated, then the receiver goes in error). The Arc doesn't need <Mutex> cause
        // the Senders implement Send and Sync traits that are needed to use this value inside a thread
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
        // iterate through Arc references and clone each one, to ensure that dispatcher() doesn't own the value, otherwise
        // it will deallocate the sender, make the receiver go in error stopping its execution
        //
        // if I had used a reference of Sender<String>, I should have created the "senders" vec with only "ch.0.clone()",
        // but then in the dispatcher() I couldn't use the thread to spawn execution, due to the fact that a thread
        // needs to own the value and doesn't accept reference (&)
        dispatcher(
            Arc::clone(sen),
            Alphanumeric.sample_string(&mut rand::thread_rng(), i + 1),
        );
    }

    // wait for all jobs to finish
    for work in worker_pool {
        work.join().unwrap()
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
                }
                Err(err) => {
                    println!("Error {err}")
                }
            }
        }
    });
}

fn create_thread_pool(max_workers: i8) -> Vec<(Sender<String>, Receiver<String>)> {
    let mut thread_pool = vec![];
    for _ in 1..max_workers {
        let ch = mpsc::channel();
        thread_pool.push(ch);
    }
    thread_pool
}

// below is a way to see how fast is Rust to execute all jobs, due to the fact that senders will be
// deallocated by it after dispatcher.dispatch(), eventually closing the receivers automatically.
// It's a round-robin scenario
pub fn run_round_robin(receiver: Receiver<JobType>) {
    let now = time::Instant::now();

    // create thread_pool for various threads and start receivers waiting for jobs
    let (worker_pool, senders) = prepare_pool();

    // create dispatcher for dispatch jobs to workers in idle
    let dispatcher = Dispatcher::new(senders);

    // waiting for jobs to come
    waiting_for_jobs(receiver, dispatcher);

    // wait for all jobs to finish
    for work in worker_pool {
        work.join().unwrap()
    }

    println!(
        "\nDuration of run_round_robin: {}\n",
        now.elapsed().as_secs_f32()
    );
}

#[rustfmt::skip]
pub fn create_jobs() -> Vec<JobType> {
    let mut jobs = vec![];
    for n in 0..MAX_JOBS_ROUND_ROBIN {
        jobs.push(
            JobType::Data(Job::new(
                format!("job: {n}"), 
                move || {
                    Ok(format!("{n}"))
                }
            ))
        )
    }
    jobs
}

fn create_thread_pool_round_robin(max_workers: i8) -> Vec<(Sender<JobType>, Receiver<JobType>)> {
    let mut thread_pool = vec![];
    for _ in 1..max_workers {
        let ch = mpsc::channel();
        thread_pool.push(ch);
    }
    thread_pool
}

pub fn prepare_pool() -> (Vec<JoinHandle<()>>, Vec<Sender<JobType>>) {
    let thread_pool = create_thread_pool_round_robin(MAX_WORKERS_ROUND_ROBIN);
    let mut num = 1;
    let mut worker_pool = vec![];
    let mut senders = vec![];

    for ch in thread_pool {
        // create a senders vec for outer dispatch operations
        senders.push(ch.0.clone());

        // create a vec to be able to wait all workers after
        worker_pool.push(thread::spawn(move || {
            let worker = Worker::new(
                ch.1,
                num,
                Alphanumeric.sample_string(&mut rand::thread_rng(), 4),
            );
            worker.run();
        }));

        num += 1;
    }

    (worker_pool, senders)
}

pub fn waiting_for_jobs(receiver: Receiver<JobType>, dispatcher: Dispatcher) {
    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(JobType::Data(job)) => {
                    // dispatch some jobs to receivers (workers)
                    dispatcher.dispatch(job);
                }
                Ok(JobType::None) => {
                    dispatcher.graceful_shutdown();
                }
                Err(err) => {
                    println!("thread waiting_for_jobs has been stopped, due to error: {err}");
                }
            }
        }
    });
}