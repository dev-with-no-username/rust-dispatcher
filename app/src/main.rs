use std::{sync::mpsc, thread, time};

use app::{create_jobs, dispatcher::JobType, run, run_round_robin};

fn main() {
    let now = time::Instant::now();

    // create channel relative to the dispatcher of job
    let (tx, rx) = mpsc::channel::<JobType>();

    // run in a separate thread all workers that will remain waiting for jobs
    let handle = thread::spawn(|| {
        run_round_robin(rx);
    });

    // send some jobs to test
    let jobs = create_jobs();
    for job in jobs {
        let _ = tx.clone().send(job);
    }

    // waiting for dispatcher to end
    handle.join().unwrap();

    // wait some seconds before running the second example
    thread::sleep(time::Duration::new(10, 0));

    run();

    println!("\nDuration of main: {}\n", now.elapsed().as_secs_f32())
}
