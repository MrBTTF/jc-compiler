use std::env;
use std::process::Command;

#[test]
fn test_hello() {
    let src = "hello";
    let output = compile_src(&src);
    assert_eq!(&output, "Hello World!\nNummer33\nSome test\n199Qwerty\n")
}

#[test]
fn test_loop() {
    let src = "loop";
    let output = compile_src(&src);
    assert_eq!(
        &output,
        "Loop starts
01234
Loop ends
"
    )
}

#[test]
fn test_assignment() {
    let src = "assignment";
    let output = compile_src(&src);
    assert_eq!(
        &output,
        "Value
33
Test
199"
    )
}

#[cfg(target_os = "linux")]
fn compile_src(src: &str) -> String {
    let dest = env::current_dir()
        .unwrap()
        .join(&format!("local/bin/{src}"));
    let src = env::current_dir()
        .unwrap()
        .join(&format!("tests/fixtures/{src}.jc"));

    let child = Command::new("cargo")
        .args(&["run", src.to_str().unwrap(), dest.to_str().unwrap()])
        .env("RUST_BACKTRACE", "1")
        .output()
        .unwrap();

    if !child.status.success() {
        panic!("{}", String::from_utf8(child.stderr).unwrap());
    }

    let stdout = Command::new(dest.to_str().unwrap())
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap()
}

#[cfg(target_os = "windows")]
fn compile_src(src: &str) -> String {
    let dest = if cfg!(target_os = "windows") {
        env::current_dir()
            .unwrap()
            .join(&format!("local/bin/{src}.exe"))
    } else {
        env::current_dir()
            .unwrap()
            .join(&format!("local/bin/{src}"))
    };
    let src = env::current_dir()
        .unwrap()
        .join(&format!("tests/fixtures/{src}.jc"));
    let child = Command::new("cargo")
        .args(&["run", src.to_str().unwrap(), dest.to_str().unwrap()])
        .env("RUST_BACKTRACE", "1")
        .output()
        .unwrap();

    if !child.status.success() {
        panic!("{}", String::from_utf8(child.stderr).unwrap());
    }
    let stdout = Command::new(dest.to_str().unwrap())
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap().replace("\r\n", "\n")
}
