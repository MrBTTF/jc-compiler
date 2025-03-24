use crate::emitter::{
    ast::{self, VarDeclarationType},
    data::{Data, DataLocation},
};

use super::super::{code_context::CodeContext, mnemonics::*};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

impl From<Data> for Arg {
    fn from(data: Data) -> Self {
        match data.data_loc {
            DataLocation::Stack(stack_loc) => Arg::Stack(u64::from(stack_loc) as i64),
            DataLocation::DataSection(data_loc) => Arg::Data(u64::from(data_loc) as i64),
        }
    }
}

pub const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(code_context: &mut CodeContext, args: &[Data]) {
    args.iter().enumerate().for_each(|(i, arg)| {
        code_context.add(PUSH.op1(ARG_REGISTERS[i]));
        match &arg.data_loc {
            DataLocation::Stack(stack_loc) => {
                let stack_loc: u32 = stack_loc.into();
                code_context.add(MOV.op1(ARG_REGISTERS[i]).op2(register::RBP));
                code_context.add(SUB.op1(ARG_REGISTERS[i]).op2(stack_loc));
            }
            DataLocation::DataSection(data_loc) => {
                code_context
                    .add(MOV.op1(ARG_REGISTERS[i]).op2(*data_loc))
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
