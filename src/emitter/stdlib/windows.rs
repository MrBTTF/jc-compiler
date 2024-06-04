use std::{collections::BTreeMap, mem};

use crate::emitter::{abi::windows::ARG_REGISTERS, data, data::DataRef, mnemonics::*, code_context::CodeContext};
use crate::emitter::ast::DeclarationType;

#[derive(Debug, Clone, Copy)]
pub enum Io {
    Stdin = 0x0,
    Stdout = 0x1,
    Stderr = 0x2,
}

fn _get_io_handle(code_context: &mut CodeContext, io_handle: Io) {
    code_context.add_slice(&[
        MOV.op1(ARG_REGISTERS[0]).op2(io_handle as u64),
        CALL.op1(Operand::Offset32(0))
            .symbol("__acrt_iob_func".to_string()),
        MOV.op1(register::R10).op2(register::RAX),
    ]);
}

fn _print(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::RDX).op2(register::R10),
        MOV.op1(register::RCX).op2(0_u64),
        MOV.op1(register::R9).op2(0_u64),
        SUB.op1(register::RSP).op2(32_u32),
        CALL.op1(Operand::Offset32(0))
            .symbol("__stdio_common_vfprintf".to_string()),
        ADD.op1(register::RSP).op2(32_u32),
    ]);
}

pub fn print(code_context: &mut CodeContext) {
    code_context.add(MOV.op1(register::R8).op2(ARG_REGISTERS[0]));
    _get_io_handle(code_context, Io::Stdout);
    _print(code_context);
}

fn _printd(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::RCX).op2(0_u64),
        MOV.op1(register::RAX).op2(ARG_REGISTERS[1]),
        MOV.op1(register::R9).op2(0_u64),
        SUB.op1(register::RSP).op2(8_u32),
        PUSH.op1(register::RAX),
        SUB.op1(register::RSP).op2(32_u32),
        MOV.op1(register::RDX).op2(register::R10),
        CALL.op1(Operand::Offset32(0))
            .symbol("__stdio_common_vfprintf".to_string()),
        ADD.op1(register::RSP).op2(48_u32),
    ]);
}

pub fn printd(code_context: &mut CodeContext) {
    code_context.add(MOV.op1(register::R8).op2(ARG_REGISTERS[0]));
    _get_io_handle(code_context, Io::Stdout);
    _printd(code_context);
}

pub fn exit(code_context: &mut CodeContext, exit_code: u64) {
    code_context.add_slice(&[
        MOV.op1(register::RAX).op2(exit_code),
        CALL.op1(Operand::Offset32(0))
            .symbol("ExitProcess".to_string()),
    ]);
}
