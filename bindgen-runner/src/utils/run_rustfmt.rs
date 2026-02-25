use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

pub fn rustfmt_huge_width(input: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let rustfmt = Command::new("rustup")
        .args(&["run", "nightly", "rustfmt"])
        .arg("--config")
        .arg("max_width=1000")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let _ = rustfmt.stdin.unwrap().write_all(input.as_ref())?;
    }
    let mut reformatted_code = String::with_capacity(4 * 1024 * 1024);
    let _ = rustfmt
        .stdout
        .unwrap()
        .read_to_string(&mut reformatted_code)?;
    Ok(reformatted_code)
}
