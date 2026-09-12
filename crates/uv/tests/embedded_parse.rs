use std::io::Write;
use std::process::{Command, ExitCode};

#[allow(unsafe_code)]
fn main() -> ExitCode {
    if let Some(argument) = std::env::args_os().nth(1) {
        // SAFETY: This executable has not started any other threads.
        let code = unsafe { uv::main(["uv".into(), argument]) };
        writeln!(std::io::stderr(), "host resumed").expect("write host marker");
        return code;
    }

    let executable = std::env::current_exe().expect("test executable path");
    for (argument, expected_code) in [("--help", 0), ("--version", 0), ("--invalid-argument", 2)] {
        let output = Command::new(&executable)
            .arg(argument)
            .env("NO_COLOR", "1")
            .output()
            .expect("spawn embedding host");
        assert_eq!(output.status.code(), Some(expected_code), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).ends_with("host resumed\n"),
            "{argument}: {output:?}"
        );
        if expected_code == 0 {
            assert!(!output.stdout.is_empty(), "{argument}: {output:?}");
        } else {
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with("error:"),
                "{argument}: {output:?}"
            );
        }
    }
    ExitCode::SUCCESS
}
