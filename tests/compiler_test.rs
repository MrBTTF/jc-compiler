use std::env;
use std::process::Command;

fn compile_src(src: &str) -> String {
    let dest = env::current_dir()
        .unwrap()
        .join(&format!("local/bin/{src}.exe"));
    let src = env::current_dir()
        .unwrap()
        .join(&format!("tests/fixtures/{src}.jc"));
    Command::new("cargo")
        .args(&["run", src.to_str().unwrap(), dest.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = Command::new(dest.to_str().unwrap())
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap()
}

#[test]
fn test_hello() {
    let src = "hello";
    let output = compile_src(&src);
    assert_eq!(&output, "Hello World!\r\nNummer33\r\nSome test\r\n199Qwerty\r\n")
}

#[test]
fn test_loop() {
    let src = "loop";
    let output = compile_src(&src);
    assert_eq!(&output, "repeat\r\nrepeat\r\nrepeat\r\nrepeat\r\nrepeat\r\ntest\r\ntest\r\ntest\r\ntest\r\ntest\r\n")
}
