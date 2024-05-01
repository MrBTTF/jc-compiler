use std::{collections::BTreeMap, mem};

use crate::emitter::{
    abi::windows::ARG_REGISTERS, data::DataRef, mnemonics::*, structs::CodeContext,
};

#[derive(Debug, Clone, Copy)]
pub enum Io {
    Stdin = 0x0,
    Stdout = 0x1,
    Stderr = 0x2,
}

fn _get_io_handle(code_context: &mut CodeContext, io_handle: Io) {
    code_context.add_slice(&[
        MOV.op1(Operand::Register(ARG_REGISTERS[0]))
            .op2(Operand::Imm64(io_handle as u64)),
        CALL.op1(Operand::Offset32(0))
            .symbol("__acrt_iob_func".to_string()),
        MOV.op1(Operand::Register(register::RDX))
            .op2(Operand::Register(register::RAX)),
    ]);
}

fn _print(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(Operand::Register(register::RCX))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::R9))
            .op2(Operand::Imm64(0)),
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
        CALL.op1(Operand::Offset32(0))
            .symbol("__stdio_common_vfprintf".to_string()),
        ADD.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
    ]);
}

pub fn print(code_context: &mut CodeContext) {
    code_context.add(
        MOV.op1(Operand::Register(register::R8))
            .op2(Operand::Register(ARG_REGISTERS[0])),
    );
    _get_io_handle(code_context, Io::Stdout);
    _print(code_context);
}

fn _printd(code_context: &mut CodeContext, number: i64) {
    code_context.add_slice(&[
        MOV.op1(Operand::Register(register::RCX))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::R9))
            .op2(Operand::Imm64(0)),
        MOV.op1(Operand::Register(register::RAX))
            .op2(Operand::Imm64(0)),
    ]);
    code_context
        .add(
            MOV.op1(Operand::Register(register::RAX))
                .op2(Operand::Imm64(0)),
        )
        .with_const_data(number.to_le_bytes().to_vec());

    code_context.add_slice(&[
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(8)),
        PUSH.op1(Operand::Register(register::RAX)),
        SUB.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(32)),
        CALL.op1(Operand::Offset32(0))
            .symbol("__stdio_common_vfprintf".to_string()),
        ADD.op1(Operand::Register(register::RSP))
            .op2(Operand::Imm32(48)),
    ]);
}

pub fn printd(code_context: &mut CodeContext, number: i64) {
    code_context.add(
        MOV.op1(Operand::Register(register::R8))
            .op2(Operand::Register(ARG_REGISTERS[0])),
    );
    _get_io_handle(code_context, Io::Stdout);
    _printd(code_context, number);
}

pub fn exit(code_context: &mut CodeContext, exit_code: i64) {
    code_context.add_slice(&[
        MOV.op1(Operand::Register(register::RAX))
            .op2(Operand::Imm64(exit_code as u64)),
        CALL.op1(Operand::Offset32(0))
            .symbol("ExitProcess".to_string()),
    ]);
}
