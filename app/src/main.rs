use std::{thread, time};

use app::{run, run_for_velocity};

fn main() {
    let now = time::Instant::now();

    run_for_velocity();

    // wait some seconds before running the second example
    thread::sleep(time::Duration::new(5, 0));
    
    run();

    println!("\nDuration of main: {}\n", now.elapsed().as_secs_f32())
}
