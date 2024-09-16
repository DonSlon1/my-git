use std::io::Write;
use std::process::{Command, Stdio};

pub fn display_with_pager(data: &str) {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());

    let mut child = Command::new(&pager)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to start pager: {}", pager));

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data.as_bytes()).expect("Failed to write to pager");
    }

    child.wait().expect("Failed to wait on pager");
}