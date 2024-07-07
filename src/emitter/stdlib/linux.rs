use abi::linux::*;

use crate::emitter::{code_context::CodeContext, *};

pub fn print(code_context: &mut CodeContext, data: Data) {
    code_context.add_slice(&[
        MOV.op1(register::RSI).op2(register::RDI),
        MOV.op1(register::RDX).op2(data.lit.len() as u64),
        MOV.op1(register::RDI).op2(STDOUT_FD),
        MOV.op1(register::RAX).op2(SYS_WRITE),
        SYSCALL.op1(05 as u8),
    ]);
}

pub fn printd(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::RSI)
            .op2(register::RSI)
            .disp(Operand::Offset32(0)),
        XOR.op1(register::RAX).op2(register::RAX), // number of vector registers
        CALL.op1(Operand::Offset32(0)).symbol("printf".to_string(), CallType::Global),
        XOR.op1(register::RDI).op2(register::RDI),
        CALL.op1(Operand::Offset32(0)).symbol("fflush".to_string(), CallType::Global),
    ]);
}

pub fn exit(code_context: &mut CodeContext, exit_code: u64) {
    code_context.add_slice(&[
        MOV.op1(register::RDI).op2(exit_code),
        MOV.op1(register::RAX).op2(SYS_EXIT),
        SYSCALL.op1(05 as u8),
    ]);
}
