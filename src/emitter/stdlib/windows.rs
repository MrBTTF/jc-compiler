use std::{collections::BTreeMap, mem};

use crate::emitter::{
    abi::windows::ARG_REGISTERS, data::DataRef, mnemonics::*, structs::Instructions,
};

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
    let result: Instructions = vec![
        MOV.op1(Operand::Register(ARG_REGISTERS[0]))
            .op2(Operand::Imm64(io_handle as u64)),
        CALL.op1(Operand::Offset32(0)),
        MOV.op1(Operand::Register(register::RDX))
            .op2(Operand::Register(register::RAX)),
    ];
    pc += result.len() - 2;
    calls.insert(pc, "__acrt_iob_func".to_string());
    result
}

fn _print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    let stdio_common_vfprintf: Instructions = vec![
        MOV.op1(Operand::Register(register::RCX))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::R9))
            .op2(Operand::Imm64(0)),
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
        CALL.op1(Operand::Offset32(0)),
        ADD.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
    ];
    pc += stdio_common_vfprintf.len() - 1 - 1;
    calls.insert(pc, "__stdio_common_vfprintf".to_string());
    stdio_common_vfprintf
}

pub fn print(calls: &mut BTreeMap<usize, String>, mut pc: usize) -> Instructions {
    let prelude: Instructions = vec![MOV
        .op1(Operand::Register(register::R8))
        .op2(Operand::Register(ARG_REGISTERS[0]))];
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
        MOV.op1(Operand::Register(register::RCX))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::R9))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::RAX))
            .op2(Operand::Imm64(0)),
    ];
    pc += stdio_common_vfprintf_1.len() - 1;
    data_refs.insert(
        pc,
        DataRef {
            offset: stdio_common_vfprintf_1.last().unwrap().get_value_loc(),
            data: number.to_le_bytes().to_vec(),
        },
    );
    let stdio_common_vfprintf_2: Instructions = vec![
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(8)),
        PUSH.op1(Operand::Register(register::RAX)),
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
        CALL.op1(Operand::Offset32(0)),
        ADD.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(48)),
    ];
    pc += stdio_common_vfprintf_2.len() - 1;
    calls.insert(pc, "__stdio_common_vfprintf".to_string());
    [stdio_common_vfprintf_1, stdio_common_vfprintf_2].concat()
}

pub fn printd(
    calls: &mut BTreeMap<usize, String>,
    data_refs: &mut BTreeMap<usize, DataRef>,
    number: i64,
    mut pc: usize,
) -> Instructions {
    let prelude: Instructions = vec![MOV
        .op1(Operand::Register(register::R8))
        .op2(Operand::Register(ARG_REGISTERS[0]))];
    pc += prelude.len();
    let _get_io_handle_code = _get_io_handle(Io::Stdout, calls, pc);
    pc += _get_io_handle_code.len();
    let _print_code = _printd(calls, data_refs, number, pc);
    [prelude, _get_io_handle_code, _print_code].concat()
}

pub fn exit(exit_code: i64, calls: &mut BTreeMap<usize, String>, pc: usize) -> Instructions {
    let result: Instructions = vec![
        MOV.op1(Operand::Register(register::RAX))
            .op2(Operand::Imm64(exit_code as u64)),
        CALL.op1(Operand::Offset32(0)),
    ];
    calls.insert(pc + result.len() - 1, "ExitProcess".to_string());
    result
}
