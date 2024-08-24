use std::io::{self, Write};

pub fn progress_bar(iteration: u64, total: u64, prefix: &str, suffix: &str) {
    let fill = '█';
    let length = 50;
    let percent = 100.0 * (iteration as f64 / total as f64);
    let filled_length = (length as f64 * iteration as f64 / total as f64).round() as usize;
    let bar = fill.to_string().repeat(filled_length) + &"-".repeat(length - filled_length);

    print!("\r{} |{}| {:.1}% {}", prefix, bar, percent, suffix);
    io::stdout().flush().unwrap();

    if iteration == total {
        println!();
    }
}