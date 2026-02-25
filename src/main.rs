use rust_game::{RunMode, run_app};

fn main() {
    run_app(parse_run_mode_from_args());
}

fn parse_run_mode_from_args() -> RunMode {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--mode=") {
            return RunMode::parse_cli_value(value).unwrap_or(RunMode::Client);
        }
        if arg == "--mode" {
            if let Some(value) = args.next() {
                return RunMode::parse_cli_value(&value).unwrap_or(RunMode::Client);
            }
            return RunMode::Client;
        }
    }

    RunMode::Client
}
