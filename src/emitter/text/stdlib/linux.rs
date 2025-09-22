use std::mem;

use crate::emitter::text::{abi::linux::*, mnemonics::*, CodeContext};

pub fn print(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        // Copy length value to RDX
        MOV.op1(ARG_REGISTERS[2])
            .op2(ARG_REGISTERS[0])
            .disp(Operand::Offset32(0)),
        // Move RDI to string pointer
        ADD.op1(ARG_REGISTERS[0]).op2(mem::size_of::<u64>() as u32),
        // Copy string pointer to RSI
        MOV.op1(ARG_REGISTERS[1]).op2(ARG_REGISTERS[0]),
    ]);

    code_context.add_slice(&[
        MOV.op1(register::RDI).op2(STDOUT_FD),
        MOV.op1(register::RAX).op2(SYS_WRITE),
        SYSCALL.op1(05 as u8),
    ]);
}

pub fn itoa(code_context: &mut CodeContext) {
    code_context.add_slice(&[
        MOV.op1(register::RAX).op2(ARG_REGISTERS[0]),
        XOR.op1(register::R9).op2(register::R9),
        XOR.op1(register::R8).op2(register::R8),
        XOR.op1(register::R11).op2(register::R11),
        MOV.op1(register::R10).op2(10_u64),
        XOR.op1(register::RDX).op2(register::RDX),
    ]);
    let loop_start = code_context.get_code_size();
    code_context.add_slice(&[
        DIV.op1(register::R10),
        ADD.op1(register::RDX).op2(0x30_u32), // ascii code for '0'
        SHL.op1(register::R8).op2(8_u8),
        OR.op1(register::R8).op2(register::RDX),
        XOR.op1(register::RDX).op2(register::RDX),
        INC.op1(register::R9),
        INC.op1(register::R11),
        CMP.op1(register::R11).op2(8_u32),
    ]);
    let skip_push = vec![
        PUSH.op1(register::R8),
        XOR.op1(register::R8).op2(register::R8),
        XOR.op1(register::R11).op2(register::R11),
    ]
    .iter_mut()
    .flat_map(|m| m.as_vec())
    .collect::<Vec<_>>()
    .len();

    code_context.add(JL.op1(Operand::Offset32(skip_push as i32)));
    code_context.add_slice(&[
        PUSH.op1(register::R8),
        XOR.op1(register::R8).op2(register::R8),
        XOR.op1(register::R11).op2(register::R11),
    ]);

    code_context.add(CMP.op1(register::RAX).op2(0_u32));
    let jump = JG.op1(Operand::Offset32(-(0 as i32))).as_vec().len() + code_context.get_code_size()
        - loop_start;
    code_context.add(JG.op1(Operand::Offset32(-(jump as i32))));

    code_context.add_slice(&[
        MOV.op1(register::RAX).op2(8_u64),
        SUB.op1(register::RAX).op2(register::R11),
        MOV.op1(register::RCX).op2(8_u64),
        MUL.op1(register::RCX),
        MOV.op1(register::RCX).op2(register::RAX), // RCX = length
        SHL_CL.op1(register::R8),                  // left shift of amount in RCX
    ]);
    code_context.add_slice(&[
        PUSH.op1(register::R8),
        MOV.op1(register::RAX).op2(register::RSP),
        MOV.op1(register::RDI).op2(8_u64),
        SUB.op1(register::RDI).op2(register::R11),
        ADD.op1(register::RAX).op2(register::RDI),
    ]);
}

pub fn printd(code_context: &mut CodeContext) {
    itoa(code_context);
    code_context.add_slice(&[
        MOV.op1(register::RSP).op2(register::RAX),
        PUSH.op1(register::R9),
        SUB.op1(register::RAX).op2(mem::size_of::<u64>() as u32), // make RAX point to length
        MOV.op1(ARG_REGISTERS[0]).op2(register::RAX),
    ]);
    print(code_context);

    code_context.add_slice(&[
        POP.op1(register::R9),
        ADD.op1(register::RSP).op2(register::R9), // pop string
    ]);
}

pub fn exit(code_context: &mut CodeContext, exit_code: u64) {
    code_context.add_slice(&[
        MOV.op1(register::RDI).op2(exit_code),
        MOV.op1(register::RAX).op2(SYS_EXIT),
        SYSCALL.op1(05 as u8),
    ]);
}
