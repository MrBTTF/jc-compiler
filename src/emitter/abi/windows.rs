use std::collections::BTreeMap;

use crate::emitter::{
    ast::{self, AssignmentType},
    data::{Data, DataRef},
};

use super::super::{mnemonics::*, structs::CodeContext};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

impl From<Data> for Arg {
    fn from(data: Data) -> Self {
        match data.assign_type {
            ast::AssignmentType::Let => Arg::Stack(data.data_loc() as i64),
            ast::AssignmentType::Const => Arg::Data(data.data_loc() as i64),
        }
    }
}
pub const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(
    code_context: &mut CodeContext,
    data_refs: &mut BTreeMap<usize, DataRef>,
    args: &[Data],
) {
    args.iter().enumerate().for_each(|(i, arg)| {
        code_context.add(PUSH.op1(Operand::Register(ARG_REGISTERS[i])));
        match arg.assign_type {
            AssignmentType::Let => {
                code_context.add(
                    MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                        .op2(Operand::Register(register::RBP)),
                );
                code_context.add(
                    SUB.op1(Operand::Register(ARG_REGISTERS[i]))
                        .op2(Operand::Imm32(arg.data_loc() as u32)),
                );
            }
            AssignmentType::Const => {
                code_context.add(
                    MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                        .op2(Operand::Imm64(arg.data_loc())),
                );
                data_refs.insert(
                    code_context.get_pc() - 1,
                    DataRef {
                        offset: code_context.last().get_value_loc(),
                        data: match &arg.lit {
                            ast::Literal::String(s) => [s.as_bytes().to_vec(), vec![0]].concat(),
                            ast::Literal::Number(_) => b"%d\0".to_vec(),
                        },
                    },
                );
            }
        }
    });
    if args.len() % 2 != 0 {
        code_context.add(
            SUB.op1(Operand::Register(register::RSP))
                .op2(Operand::Imm32(8)),
        );
    }
}

pub fn pop_args(code_context: &mut CodeContext, args_count: usize) {
    if args_count % 2 != 0 {
        code_context.add(
            ADD.op1(Operand::Register(register::RSP))
                .op2(Operand::Imm32(8)),
        );
    }
    (0..args_count).for_each(|i| {
        code_context.add(POP.op1(Operand::Register(ARG_REGISTERS[i])));
    });
}
