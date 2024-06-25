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
    let length = 2i32; //TODO: calculate in asm or call printf from C

    let to_dec = (0..length).fold(vec![], |mut acc, _i| {
        acc.extend(vec![
            // xor rdx, rdx
            XOR.op1(register::RDX).op2(register::RDX),
            // div rcx
            DIV.op1(register::RCX),
            // add rdx, '0'
            ADD.op1(register::RDX).op2('0' as u32),
            // shl rbx, 8
            SHL.op1(register::RBX).op2(8 as u8),
            // or rbx, rdx
            OR.op1(register::RBX).op2(register::RDX),
        ]);

        acc
    });

    code_context.add_slice(&[
        MOV.op1(register::RAX)
            .op2(register::RSI)
            .disp(Operand::Offset32(0)),
        MOV.op1(register::RCX).op2(10 as u64),
        XOR.op1(register::RBX).op2(register::RBX),
    ]);
    code_context.add_slice(to_dec.as_slice());
    code_context.add_slice(&[
        PUSH.op1(register::RBX),
        MOV.op1(register::RSI).op2(register::RSP),
        MOV.op1(register::RDX).op2(length as u64),
        MOV.op1(register::RDI).op2(STDOUT_FD),
        MOV.op1(register::RAX).op2(SYS_WRITE),
        SYSCALL.op1(05 as u8),
        POP.op1(register::RBX),
    ]);
}

pub fn exit(code_context: &mut CodeContext, exit_code: u64) {
    code_context.add_slice(&[
        MOV.op1(register::RDI).op2(exit_code),
        MOV.op1(register::RAX).op2(SYS_EXIT),
        SYSCALL.op1(05 as u8),
    ]);
}
