use crate::emitter::stack::StackManager;
use crate::emitter::variables::{ValueLocation, Variable};

use super::super::{code_context::CodeContext, mnemonics::*};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

impl From<Variable> for Arg {
    fn from(data: Variable) -> Self {
        match data.value_loc {
            ValueLocation::Stack(stack_loc) => Arg::Stack(u64::from(stack_loc) as i64),
            ValueLocation::DataSection(data_loc) => Arg::Data(u64::from(data_loc) as i64),
        }
    }
}

pub const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(code_context: &mut CodeContext, stack: &mut StackManager, args: &[Variable]) {
    args.iter().enumerate().for_each(|(i, _)| {
        code_context.add_slice(&stack.push_register(ARG_REGISTERS[i]));
    });

    args.iter().enumerate().for_each(|(i, arg)| {
        match &arg.value_loc {
            ValueLocation::Stack(stack_loc) => {
                let stack_loc: u32 = stack_loc.into();
                code_context.add_slice(&[
                    MOV.op1(ARG_REGISTERS[i]).op2(register::RBP),
                    SUB.op1(ARG_REGISTERS[i]).op2(stack_loc),
                ]);
            }
            ValueLocation::DataSection(_) => {
                code_context.add(
                    MOV.op1(ARG_REGISTERS[i])
                        .op2(0_u64)
                        .symbol(arg.name.clone()),
                );
            }
        }

        if !arg.reference {
            code_context.add(
                MOV.op1(ARG_REGISTERS[i])
                    .op2(ARG_REGISTERS[i])
                    .disp(Operand::Offset32(0)),
            );
        }
    });
}

pub fn pop_args(code_context: &mut CodeContext, stack: &mut StackManager, args_count: usize) {
    (0..args_count).rev().for_each(|i| {
        code_context
            .add_slice(&stack.pop_register(crate::emitter::text::abi::linux::ARG_REGISTERS[i]));
    });
}
