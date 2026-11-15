mod cli;
mod runner;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match cli::parse(&args) {
        Ok(cli) => cli,
        Err(message) => exit_with_error(&message),
    };

    if let Err(error) = runner::run(&cli) {
        exit_with_error(&format!("failed to simulate match: {}", error));
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}
