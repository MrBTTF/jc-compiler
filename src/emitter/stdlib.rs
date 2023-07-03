use super::amd64::*;

pub fn print(length: usize) -> Vec<u8> {
    [
        Mov64rr::build(Register::Si, Register::Ax),
        Mov32::build(Register::Dx, length as i32),
        Mov32::build(Register::Di, STDOUT_FD),
        Mov32::build(Register::Ax, SYS_WRITE),
        SysCall::build(),
    ]
    .concat()
}

pub fn exit(exit_code: i32) -> Vec<u8> {
    [
        Mov32::build(Register::Di, exit_code),
        Mov32::build(Register::Ax, SYS_EXIT),
        SysCall::build(),
    ]
    .concat()
}
