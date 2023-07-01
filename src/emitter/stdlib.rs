use super::amd64::*;

pub fn print(length: usize) -> Vec<u8> {
    [
        Mov32rr::build(Register::Ecx, Register::Eax),
        Mov32::build(Register::Edx, length as i32),
        Mov32::build(Register::Ebx, STDOUT_FD),
        Mov32::build(Register::Eax, SYS_WRITE),
        SysCall::build(),
    ]
    .concat()
}
