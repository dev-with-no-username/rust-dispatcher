use std::time;

use app::{create_jobs_and_test, execute, init};

fn main() {
    let now = time::Instant::now();

    // create channel relative to the dispatcher of job
    let (tx, rx) = init();

    // run in a separate thread all workers that will remain waiting for jobs
    let handle = execute(rx);

    // create and send some jobs to test
    create_jobs_and_test(tx);

    // waiting for dispatcher channel to end
    handle.join().unwrap();

    println!("\nDuration of main: {}\n", now.elapsed().as_secs_f32())
}
