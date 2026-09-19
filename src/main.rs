use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::ExitCode;
use wmic_rs::cli::Destination;

#[cfg(windows)]
use wmic_rs::windows_backend::WindowsWmi as PlatformBackend;

#[cfg(not(windows))]
struct PlatformBackend;

#[cfg(not(windows))]
impl wmic_rs::Backend for PlatformBackend {
    fn query(
        &self,
        _namespace: &str,
        _node: &str,
        _wql: &str,
    ) -> Result<Vec<wmic_rs::Record>, String> {
        Err("WMI is available only on Windows".into())
    }
}

fn main() -> ExitCode {
    let invocation = match wmic_rs::parse(std::env::args_os().skip(1).map(|s| s.to_string_lossy().into_owned())) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    let execution = wmic_rs::execute(&invocation, &PlatformBackend);
    if !execution.stderr.is_empty() {
        let _ = io::stderr().write_all(execution.stderr.as_bytes());
    }
    if let Err(error) = write_output(&invocation.globals.destination, &execution.stdout) {
        eprintln!("ERROR:\r\nDescription = {error}");
        return ExitCode::from(1);
    }
    ExitCode::from(execution.exit_code as u8)
}

fn write_output(destination: &Destination, value: &str) -> io::Result<()> {
    match destination {
        Destination::Stdout => io::stdout().write_all(value.as_bytes()),
        Destination::File { path, append } => OpenOptions::new()
            .create(true)
            .write(true)
            .append(*append)
            .truncate(!append)
            .open(path)?
            .write_all(value.as_bytes()),
        Destination::Clipboard { .. } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "clipboard output is not implemented yet",
        )),
    }
}
