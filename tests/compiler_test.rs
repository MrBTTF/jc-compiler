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
Number: 0
Number: 1
Number: 2
Number: 3
Number: 4
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
Some value
199"
    )
}

#[test]
fn test_funcs() {
    let src = "funcs";
    let output = compile_src(&src);
    assert_eq!(
        &output,
        "before f1
f1: param
f1: const
33
f1: let
after f1
199
before f2
f2: param
f2: const
f2: let 1
f3: param
33
f2: let 2
f2: let 3\n"
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
