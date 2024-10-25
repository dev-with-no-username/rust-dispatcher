use std::{
    error, sync::mpsc::{self, Receiver, Sender}, thread::{self, JoinHandle}, time
};

use dispatcher::{Dispatcher, Job, JobType};
use rand::distributions::{Alphanumeric, DistString};
use worker::Worker;

pub mod dispatcher;
pub mod worker;

const MAX_JOBS: i32 = 2000000;
const MAX_WORKERS: i8 = 20;

/// create the dispatcher channel for incoming job requests
pub fn init() -> (Sender<JobType>, Receiver<JobType>) {
    let (tx, rx) = mpsc::channel::<JobType>();
    (tx, rx)
}

/// create worker pool to start waiting for job requests
pub fn execute(rx: Receiver<JobType>) -> JoinHandle<()> {
    thread::spawn(|| {
        run_round_robin(rx);
    })
}

/// create a job to be sent to dispatcher channel
pub fn create_job(name: String, function: impl Fn() -> Result<String, Box<dyn error::Error>> + Send + 'static) -> JobType {
    JobType::Data(
        Job::new(
            name, 
            function
        )
    )
}

/// gracefully shutdown the dispatcher channel and the worker pool
pub fn stop(tx: Sender<JobType>) {
    match tx.send(JobType::None) {
        Ok(_) => {},
        Err(err) => {
            println!("stop execution goes wrong, due to error: {err}")
        },
    }
}

/// actually create worker pool and start waiting for incoming job requests
fn run_round_robin(receiver: Receiver<JobType>) {
    let now = time::Instant::now();

    // create thread_pool for various threads and start receivers waiting for jobs
    let (worker_pool, senders) = create_thread_pool();

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

fn create_thread_pool() -> (Vec<JoinHandle<()>>, Vec<Sender<JobType>>) {
    let worker_pool = create_worker_pool(MAX_WORKERS);
    let mut num = 1;
    let mut thread_pool = vec![];
    let mut senders = vec![];

    for ch in worker_pool {
        // create a senders vec for outer dispatch operations, otherwise they will be
        // deallocated once exit the for scope and eventually we are not able to
        // dispatch jobs to workers receivers that are mandatory connects to senders
        senders.push(ch.0);

        // create a vec to be able to wait all workers finish
        thread_pool.push(thread::spawn(move || {
            let worker = Worker::new(
                ch.1,
                num,
                Alphanumeric.sample_string(&mut rand::thread_rng(), 4),
            );
            worker.run();
        }));

        num += 1;
    }

    (thread_pool, senders)
}

fn create_worker_pool(max_workers: i8) -> Vec<(Sender<JobType>, Receiver<JobType>)> {
    let mut worker_pool = vec![];
    for _ in 0..max_workers {
        let ch = mpsc::channel();
        worker_pool.push(ch);
    }
    worker_pool
}

fn waiting_for_jobs(receiver: Receiver<JobType>, dispatcher: Dispatcher) {
    let mut index = 0;
    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(JobType::Data(job)) => {
                    // dispatch some jobs to receivers (workers)
                    dispatcher.dispatch(job, index);
                    if index == (MAX_WORKERS - 1).try_into().unwrap() {
                        index = 0
                    } else {
                        index += 1
                    }
                }
                Ok(JobType::None) => {
                    dispatcher.graceful_shutdown();
                }
                Err(_) => {
                    // println!("thread waiting_for_jobs has been stopped, due to error: {err}");
                }
            }
        }
    });
}

#[rustfmt::skip]
pub fn create_jobs_and_test(tx: Sender<JobType>) {
    let mut jobs = vec![];
    for n in 0..MAX_JOBS {
        jobs.push(
            JobType::Data(Job::new(
                format!("job: {n}"), 
                move || {
                    Ok(format!("{n}"))
                }
            ))
        )
    }
    let mut num = 0;
    let len = jobs.len();
    for job in jobs {
        if num == len - 1 {
            stop(tx);
            break;
        } else {
            num += 1;
            let _ = tx.clone().send(job);
        }
    }
}