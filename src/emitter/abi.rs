use std::env::Args;

use super::amd64::*;

#[derive(Debug)]
pub enum Arg {
    Data(i32),
    Stack(i32),
}

const arg_registers: &[Register] = &[Register::Eax];

pub fn push_args(args: &[Arg]) -> Vec<u8> {
    args.iter().enumerate().fold(vec![], |mut acc, (i, arg)| {
        let arg_reg = arg_registers[i];
        acc.extend(Push32rr::build(arg_reg));
        match arg {
            Arg::Stack(v) => {
                acc.extend(Mov32rr::build(arg_reg, Register::Ebp));
                acc.extend(Sub32::build(arg_reg, *v));
            }
            Arg::Data(v) => acc.extend(Mov32::build(arg_reg, *v)),
        }
        acc
    })
}

pub fn pop_args(args_count: usize) -> Vec<u8> {
    arg_registers[..args_count]
        .iter()
        .fold(vec![], |mut acc, arg_reg| {
            acc.extend(Pop32rr::build(*arg_reg));
            acc
        })
}
