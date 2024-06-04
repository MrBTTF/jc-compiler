use std::collections::BTreeMap;

use crate::emitter::{
    ast::{self, DeclarationType},
    data::{Data, DataRef},
};
use crate::emitter::ast::Literal;

use super::super::{mnemonics::*, code_context::CodeContext};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

impl From<Data> for Arg {
    fn from(data: Data) -> Self {
        match data.decl_type {
            ast::DeclarationType::Let => Arg::Stack(data.data_loc() as i64),
            ast::DeclarationType::Const => Arg::Data(data.data_loc() as i64),
        }
    }
}
pub const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(
    code_context: &mut CodeContext,
    args: &[Data],
) {
    args.iter().enumerate().for_each(|(i, arg)| {
        code_context.add(PUSH.op1(ARG_REGISTERS[i]));
        match arg.decl_type {
            DeclarationType::Let => {
                code_context.add(
                    MOV.op1(ARG_REGISTERS[i])
                        .op2(register::RBP),
                );
                code_context.add(
                    SUB.op1(ARG_REGISTERS[i])
                        .op2(arg.data_loc() as u32),
                );
            }
            DeclarationType::Const => {
                code_context
                    .add(
                        MOV.op1(ARG_REGISTERS[i])
                            .op2(arg.data_loc()),
                    )
                    .with_const_data(arg.lit.as_vec());
            }
        }
    });
    if args.len() % 2 != 0 {
        code_context.add(
            SUB.op1(register::RSP)
                .op2(8_u32),
        );
    }
}

pub fn pop_args(code_context: &mut CodeContext, args_count: usize) {
    if args_count % 2 != 0 {
        code_context.add(
            ADD.op1(register::RSP)
                .op2(8_u32),
        );
    }
    (0..args_count).for_each(|i| {
        code_context.add(POP.op1(ARG_REGISTERS[i]));
    });
}
