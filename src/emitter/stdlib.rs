use super::{amd64::*, structs::Instruction, structs::Instructions};

pub fn print(length: usize) -> Instructions {
    vec![
        Mov64rr::new(Register::Si, Register::Ax) as Box<dyn Instruction>,
        Mov32::new(Register::Dx, length as i32),
        Mov32::new(Register::Di, STDOUT_FD),
        Mov32::new(Register::Ax, SYS_WRITE),
        SysCall::new(),
    ]
}

pub fn printd() -> Instructions {
    let length = 3i32;

    let to_dec = (0..length).fold(vec![], |mut acc, _i| {
        acc.extend(vec![
            // xor rdx, rdx
            Xor32rr::new(Register::Dx, Register::Dx) as Box<dyn Instruction>,
            // div rcx
            Div32::new(Register::Cx),
            // add rdx, '0'
            Add32::new(Register::Dx, '0' as i32),
            // shl rbx, 8
            Shl32::new(Register::Bx, 8),
            // or rbx, rdx
            Or32rr::new(Register::Bx, Register::Dx),
        ]);

        acc
    });

    let mut result = vec![];
    // Sub64::build(Register::Ax, 4),
    result.extend(vec![
        Mov64Ref::new(Register::Ax, Register::Ax, 0) as Box<dyn Instruction>,
        Mov32::new(Register::Cx, 10),
        Xor32rr::new(Register::Bx, Register::Bx),
    ]);
    result.extend(to_dec);
    result.extend(vec![
        Push::new(Register::Bx) as Box<dyn Instruction>,
        Mov64rr::new(Register::Si, Register::Sp),
        Mov32::new(Register::Dx, length),
        Mov32::new(Register::Di, STDOUT_FD),
        Mov32::new(Register::Ax, SYS_WRITE),
        SysCall::new(),
        Pop::new(Register::Bx),
    ]);
    result
}

pub fn exit(exit_code: i32) -> Instructions {
    vec![
        Mov32::new(Register::Di, exit_code),
        Mov32::new(Register::Ax, SYS_EXIT),
        SysCall::new(),
    ]
}
