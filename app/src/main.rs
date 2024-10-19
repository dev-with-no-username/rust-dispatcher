use std::time;

use app::run;

fn main() {
    let now = time::Instant::now();

    run();

    println!("{}", now.elapsed().as_secs_f32())
}
