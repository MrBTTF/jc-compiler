use std::collections::BTreeMap;

use crate::emitter::{
    ast::{self, AssignmentType},
    data::{Data, DataRef},
};

use super::super::{mnemonics::*, structs::Instructions};

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
    pc: usize,
    data_refs: &mut BTreeMap<usize, DataRef>,
    args: &[Data],
) -> Instructions {
    let mut result = args
        .iter()
        .enumerate()
        .fold(vec![], |mut acc: Instructions, (i, arg)| {
            acc.push(PUSH.op1(Operand::Register(ARG_REGISTERS[i])));
            let pc = pc + 1;
            match arg.assign_type {
                AssignmentType::Let => {
                    acc.push(
                        MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                            .op2(Operand::Register(register::RBP)),
                    );
                    acc.push(
                        SUB.op1(Operand::Register(ARG_REGISTERS[i]))
                            .op2(Operand::Imm32(arg.data_loc() as u32)),
                    );
                }
                AssignmentType::Const => {
                    acc.push(
                        MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                            .op2(Operand::Imm64(arg.data_loc())),
                    );
                    data_refs.insert(
                        pc,
                        DataRef {
                            offset: acc.last().unwrap().get_value_loc(),
                            data: match &arg.lit {
                                ast::Literal::String(s) => {
                                    [s.as_bytes().to_vec(), vec![0]].concat()
                                }
                                ast::Literal::Number(_) => b"%d\0".to_vec(),
                            },
                        },
                    );
                }
            }
            acc
        });
    if args.len() % 2 != 0 {
        result.push(
            SUB.op1(Operand::Register(register::RSP))
                .op2(Operand::Imm32(8)),
        );
    }
    result
}

pub fn pop_args(args_count: usize) -> Instructions {
    let mut result = vec![];
    if args_count % 2 != 0 {
        result.push(
            ADD.op1(Operand::Register(register::RSP))
                .op2(Operand::Imm32(8)),
        );
    }
    result.extend((0..args_count).fold(vec![], |mut acc: Instructions, i| {
        acc.push(POP.op1(Operand::Register(ARG_REGISTERS[i])));
        acc
    }));
    result
}
