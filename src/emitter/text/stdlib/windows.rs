use std::mem;

use crate::emitter::{text::abi::windows::ARG_REGISTERS, text::mnemonics::*, text::CodeContext};

use crate::emitter::variables::Variable;

#[derive(Debug, Clone, Copy)]
pub enum Io {
    Stdin = 0x0,
    Stdout = 0x1,
    Stderr = 0x2,
}

#[derive(Debug, Clone, Copy)]
pub enum StdHandle {
    Stdin = -10,
    Stdout = -11,
    Stderr = -12,
}

fn _get_std_handle(code_context: &mut CodeContext, std_handle: StdHandle) {
    code_context.add_slice(&[
        MOV.op1(ARG_REGISTERS[0]).op2(std_handle as u64),
        CALL.op1(Operand::Offset32(0))
            .symbol("GetStdHandle".to_string()),
        MOV.op1(register::R10).op2(register::RAX),
    ]);
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
        MOV.op1(register::RCX).op2(register::R10),
        MOV.op1(register::RDX).op2(register::R8),
        MOV.op1(register::R8)
            .op2(register::R9)
            .disp(Operand::Offset32(0)),
        SUB.op1(register::RSP).op2(48_u32),
        MOV.op1(register::R9).op2(register::RSP), // Store written bytes
        SUB.op1(register::R9).op2(16_u32),
        CALL.op1(Operand::Offset32(0))
            .symbol("WriteFile".to_string()),
        ADD.op1(register::RSP).op2(48_u32),
    ]);
}

pub fn print(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::R9).op2(ARG_REGISTERS[0]), // string length
        MOV.op1(register::R8).op2(register::R9),
        ADD.op1(register::R8).op2(mem::size_of::<u64>() as u32), // skip string length
    ]);
    _get_std_handle(code_context, StdHandle::Stdout);
    _print(code_context);
}

fn _printd(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::RCX).op2(0_u64),
        MOV.op1(register::RAX).op2(ARG_REGISTERS[1]),
        MOV.op1(register::R9).op2(0_u64),
        // Pushing the value and its adress to the stack
        PUSH.op1(register::RAX),
        MOV.op1(register::RAX).op2(register::RSP),
        PUSH.op1(register::RAX),
        SUB.op1(register::RSP).op2(32_u32),
        MOV.op1(register::RDX).op2(register::R10),
        CALL.op1(Operand::Offset32(0))
            .symbol("__stdio_common_vfprintf".to_string()),
        ADD.op1(register::RSP).op2(48_u32), // 32 shadow space + 16 for variable on stack
        XOR.op1(register::RCX).op2(register::RCX),
        CALL.op1(Operand::Offset32(0)).symbol("fflush".to_string()),
    ]);
}

pub fn printd(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::R8).op2(ARG_REGISTERS[0]),
        ADD.op1(register::R8).op2(mem::size_of::<u64>() as u32), // string length
    ]);
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
