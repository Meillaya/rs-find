use std::process::{Command, Stdio};
use std::time::Instant;

fn main() {
    let root = match std::env::var("BENCH_ROOT") {
        Ok(root) => root,
        Err(_) => {
            println!("Set BENCH_ROOT and optionally BENCH_QUERY to compare rs-find against fd.");
            return;
        }
    };
    let query = std::env::var("BENCH_QUERY").unwrap_or_else(|_| "rs".to_owned());
    let iterations = std::env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(3);

    let rs_find_binary = std::env::var("CARGO_BIN_EXE_rs-find")
        .unwrap_or_else(|_| "target/release/rs-find".to_owned());

    println!("Benchmark root: {root}");
    println!("Benchmark query: {query}");
    println!("Iterations: {iterations}");

    let rs_find_times = run_command(iterations, || {
        let start = Instant::now();
        let status = Command::new(&rs_find_binary)
            .arg(&query)
            .arg(&root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to run rs-find benchmark command");
        assert!(status.success(), "rs-find benchmark command failed");
        start.elapsed()
    });

    println!("rs-find median: {:.2?}", median(&rs_find_times));

    match Command::new("fd").arg("--version").status() {
        Ok(status) if status.success() => {
            let fd_times = run_command(iterations, || {
                let start = Instant::now();
                let status = Command::new("fd")
                    .arg(&query)
                    .arg(&root)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .expect("failed to run fd benchmark command");
                assert!(status.success(), "fd benchmark command failed");
                start.elapsed()
            });
            println!("fd median: {:.2?}", median(&fd_times));
        }
        _ => println!("fd not installed; skipping comparison"),
    }
}

fn run_command<F>(iterations: usize, mut run: F) -> Vec<std::time::Duration>
where
    F: FnMut() -> std::time::Duration,
{
    (0..iterations).map(|_| run()).collect()
}

fn median(times: &[std::time::Duration]) -> std::time::Duration {
    let mut times = times.to_vec();
    times.sort();
    times[times.len() / 2]
}
