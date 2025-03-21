use crate::emitter::{
    ast::{self, VarDeclarationType},
    data::Data,
};

use super::super::{code_context::CodeContext, mnemonics::*};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

impl From<Data> for Arg {
    fn from(data: Data) -> Self {
        match data.decl_type {
            ast::VarDeclarationType::Let => Arg::Stack(data.data_loc as i64),
            ast::VarDeclarationType::Const => Arg::Data(data.data_loc as i64),
        }
    }
}
pub const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(code_context: &mut CodeContext, args: &[Data]) {
    args.iter().enumerate().for_each(|(i, arg)| {
        code_context.add(PUSH.op1(ARG_REGISTERS[i]));
        match arg.decl_type {
            VarDeclarationType::Let => {
                code_context.add(MOV.op1(ARG_REGISTERS[i]).op2(register::RBP));
                code_context.add(SUB.op1(ARG_REGISTERS[i]).op2(arg.data_loc as u32));
            }
            VarDeclarationType::Const => {
                code_context
                    .add(MOV.op1(ARG_REGISTERS[i]).op2(arg.data_loc))
                    .with_const_data(&arg.symbol, arg.as_vec());
            }
        }
    });
    if args.len() % 2 != 0 {
        code_context.add(SUB.op1(register::RSP).op2(8_u32));
    }
}

pub fn pop_args(code_context: &mut CodeContext, args_count: usize) {
    if args_count % 2 != 0 {
        code_context.add(ADD.op1(register::RSP).op2(8_u32));
    }
    (0..args_count).for_each(|i| {
        code_context.add(POP.op1(ARG_REGISTERS[i]));
    });
}
