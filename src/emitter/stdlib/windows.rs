use std::{collections::BTreeMap, mem};

use crate::emitter::{amd64::*, data::DataRef, structs::Instructions};

#[derive(Debug, Clone, Copy)]
pub enum Io {
    Stdin = 0x0,
    Stdout = 0x1,
    Stderr = 0x2,
}

fn _get_io_handle(
    io_handle: Io,
    calls: &mut BTreeMap<usize, String>,
    mut pc: usize,
) -> Instructions {
    let result: Instructions = vec![Mov32::new(Register::Cx, io_handle as i32), Call::new(0x0)];
    pc += result.len() - 1;
    calls.insert(pc, "__acrt_iob_func".to_string());
    result
}

fn _print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    let stdio_common_vfprintf: Instructions = vec![
        Mov64rr::new(Register::Bx, Register::Ax),
        Mov32::new(Register::Cx, 0x0),
        Mov64rr::new(Register::Dx, Register::Bx),
        Mov32Ext::new(RegisterExt::R9, 0x0),
        Push::new(Register::Bp),
        Mov64rr::new(Register::Bp, Register::Sp),
        Sub64::new(Register::Sp, 48),
        Call::new(0x0),
        Add64::new(Register::Sp, 48),
        Pop::new(Register::Bp),
    ];
    pc += stdio_common_vfprintf.len() - 1 - 2;
    calls.insert(pc, "__stdio_common_vfprintf".to_string());
    stdio_common_vfprintf
}

pub fn print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    let prelude: Instructions = vec![Mov64rExtr::new(RegisterExt::R8, Register::Cx)];
    pc += prelude.len();
    let _get_io_handle_code = _get_io_handle(Io::Stdout, calls, pc);
    pc += _get_io_handle_code.len();
    let _print_code = _print(calls, pc);
    [prelude, _get_io_handle_code, _print_code].concat()
}

fn _printd(
    calls: &mut BTreeMap<usize, String>,
    data_refs: &mut BTreeMap<usize, DataRef>,
    number: i64,
    mut pc: usize,
) -> Instructions {
    let stdio_common_vfprintf_1: Instructions = vec![
        Mov64rr::new(Register::Bx, Register::Ax),
        Mov32::new(Register::Cx, 0x0),
        Mov64rr::new(Register::Dx, Register::Bx),
        Mov32Ext::new(RegisterExt::R9, 0x0),
        Push::new(Register::Bp),
        Mov64rr::new(Register::Bp, Register::Sp),
        Sub64::new(Register::Sp, 8),
        Mov64Long::new(Register::Ax, 0),
    ];
    pc += stdio_common_vfprintf_1.len() - 1;
    data_refs.insert(
        pc,
        DataRef {
            offset: 2,
            ref_len: mem::size_of::<i64>(),
            data: number.to_le_bytes().to_vec(),
        },
    );
    let stdio_common_vfprintf_2: Instructions = vec![
        Push::new(Register::Ax),
        Sub64::new(Register::Sp, 32),
        Call::new(0x0),
        Add64::new(Register::Sp, 48),
        Pop::new(Register::Bp),
    ];
    pc += stdio_common_vfprintf_2.len() - 1 - 1;
    calls.insert(pc, "__stdio_common_vfprintf".to_string());
    [stdio_common_vfprintf_1, stdio_common_vfprintf_2].concat()
}

pub fn printd(
    calls: &mut BTreeMap<usize, String>,
    data_refs: &mut BTreeMap<usize, DataRef>,
    number: i64,
    mut pc: usize,
) -> Instructions {
    let prelude: Instructions = vec![Mov64rExtr::new(RegisterExt::R8, Register::Cx)];
    pc += prelude.len();
    let _get_io_handle_code = _get_io_handle(Io::Stdout, calls, pc);
    pc += _get_io_handle_code.len();
    let _print_code = _printd(calls, data_refs, number, pc);
    [prelude, _get_io_handle_code, _print_code].concat()
}

pub fn exit(exit_code: i64, calls: &mut BTreeMap<usize, String>, pc: usize) -> Instructions {
    let result: Instructions = vec![Mov64::new(Register::Ax, exit_code), Call::new(0x0)];
    calls.insert(pc + result.len() - 1, "ExitProcess".to_string());
    result
}
