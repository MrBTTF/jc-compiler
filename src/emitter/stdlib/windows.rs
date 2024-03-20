use std::{collections::BTreeMap, rc::Rc};

use crate::emitter::{
    amd64::*,
    structs::{Instruction, Instructions},
};

pub enum Io {
    Stdin = 0x0,
    Stdout = 0x1,
    Stderr = 0x2,
}

fn shadow_space() -> Instructions {
    vec![
        Push::new(Register::Bp),
        Mov64rr::new(Register::Bp, Register::Sp),
        Sub64::new(Register::Sp, 32),
    ]
}
fn _get_io_handle(io_handle: Io, calls: &mut BTreeMap<usize, String>, pc: usize) -> Instructions {
    calls.insert(pc + 2, "__acrt_iob_func".to_string());

    vec![
        Mov64rExtr::new(RegisterExt::R8, Register::Cx),
        Mov32::new(Register::Cx, io_handle as i32),
        Call::new(0x0),
    ]
}

fn _print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    let stdio_common_vfprintf: Instructions = vec![
        Mov64rr::new(Register::Bx, Register::Ax),
        Mov32::new(Register::Cx, 0x0),
        Mov64rr::new(Register::Dx, Register::Bx),
        Mov32Ext::new(RegisterExt::R9, 0x0),
        Call::new(0x0),
    ];
    pc += stdio_common_vfprintf.len() - 1;
    calls.insert(pc, "__stdio_common_vfprintf".to_string());
    stdio_common_vfprintf
}

pub fn print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    pc += 3;
    let _get_io_handle_code = _get_io_handle(Io::Stdout, calls, pc);
    pc += _get_io_handle_code.len();
    pc += 3;
    let _print_code = _print(calls, pc);
    [
        shadow_space(),
        _get_io_handle_code,
        shadow_space(),
        _print_code,
    ]
    .concat()
}

pub fn printd() -> Instructions {
    let length = 3i32;

    let to_dec = (0..length).fold(vec![], |mut acc, _i| {
        acc.extend(vec![
            // xor rdx, rdx
            Xor32rr::new(Register::Dx, Register::Dx) as Rc<dyn Instruction>,
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
        Mov64Ref::new(Register::Ax, Register::Ax, 0) as Rc<dyn Instruction>,
        Mov32::new(Register::Cx, 10),
        Xor32rr::new(Register::Bx, Register::Bx),
    ]);
    result.extend(to_dec);
    result.extend(vec![
        Push::new(Register::Bx) as Rc<dyn Instruction>,
        Mov64rr::new(Register::Si, Register::Sp),
        Mov32::new(Register::Dx, length),
        Mov32::new(Register::Di, STDOUT_FD),
        Mov32::new(Register::Ax, SYS_WRITE),
        SysCall::new(),
        Pop::new(Register::Bx),
    ]);
    result
}

pub fn exit(exit_code: i64, calls: &mut BTreeMap<usize, String>, pc: usize) -> Instructions {
    let result: Instructions = vec![Mov64::new(Register::Ax, exit_code), Call::new(0x0)];
    calls.insert(pc + result.len() - 1, "ExitProcess".to_string());
    result
}
