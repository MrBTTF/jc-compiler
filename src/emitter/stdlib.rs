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

pub fn printd() -> Vec<u8> {
    let length = 3i32;

    let to_dec = (0..length).fold(vec![], |mut acc, i| {
        acc.extend(
            [
                // xor rdx, rdx
                Xor32rr::build(Register::Dx, Register::Dx),
                // div rcx
                Div32::build(Register::Cx),
                // add rdx, '0'
                Add32::build(Register::Dx, '0' as i32),
                // shl rbx, 8
                Shl32::build(Register::Bx, 8),
                // or rbx, rdx
                Or32rr::build(Register::Bx, Register::Dx),
            ]
            .concat(),
        );

        acc
    });

    [
        // Sub64::build(Register::Ax, 4),
        Mov64Ref::build(Register::Ax, Register::Ax, 0),
        Mov32::build(Register::Cx, 10),
        Xor32rr::build(Register::Bx, Register::Bx),
        to_dec,
        Push::build(Register::Bx),
        Mov64rr::build(Register::Si, Register::Sp),
        Mov32::build(Register::Dx, length),
        Mov32::build(Register::Di, STDOUT_FD),
        Mov32::build(Register::Ax, SYS_WRITE),
        SysCall::build(),
        Pop::build(Register::Bx),
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
