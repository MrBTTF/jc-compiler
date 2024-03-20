use std::rc::Rc;

use crate::emitter::structs::Instruction;

use super::super::{amd64::*, structs::Instructions};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

const ARG_REGISTERS: &[Register] = &[Register::Cx, Register::Dx];
const ARG_REGISTERS_EXT: &[RegisterExt] = &[RegisterExt::R8, RegisterExt::R9];

fn push_reg(i: usize) -> Rc<dyn Instruction> {
    if i < 2 {
        let arg_reg = ARG_REGISTERS[i];
        Push::new(arg_reg)
    } else if i >= 2 {
        let arg_reg = ARG_REGISTERS_EXT[i - 2];
        PushExt::new(arg_reg)
    } else {
        panic!("Too much arguments (>4)");
    }
}

fn pop_reg(i: usize) -> Rc<dyn Instruction> {
    if i < 2 {
        let arg_reg = ARG_REGISTERS[i];
        Pop::new(arg_reg)
    } else if i >= 2 {
        let arg_reg = ARG_REGISTERS_EXT[i - 2];
        PopExt::new(arg_reg)
    } else {
        panic!("Too much arguments (>4)");
    }
}

fn mov_rr(i: usize, reg: Register) -> Rc<dyn Instruction> {
    if i < 2 {
        let arg_reg = ARG_REGISTERS[i];
        Mov64rr::new(arg_reg, reg)
    } else if i >= 2 {
        let arg_reg = ARG_REGISTERS_EXT[i - 2];
        Mov64rExtr::new(arg_reg, reg)
    } else {
        panic!("Too much arguments (>4)");
    }
}

fn mov_value(i: usize, value: i64) -> Rc<dyn Instruction> {
    if i < 2 {
        let arg_reg = ARG_REGISTERS[i];
        Mov64::new(arg_reg, value)
    } else if i >= 2 {
        let arg_reg = ARG_REGISTERS_EXT[i - 2];
        Mov64Ext::new(arg_reg, value)
    } else {
        panic!("Too much arguments (>4)");
    }
}

fn sub_reg(i: usize, value: i32) -> Rc<dyn Instruction> {
    if i < 2 {
        let arg_reg = ARG_REGISTERS[i];
        Sub64::new(arg_reg, value)
    } else if i >= 2 {
        let arg_reg = ARG_REGISTERS_EXT[i - 2];
        Sub64Ext::new(arg_reg, value)
    } else {
        panic!("Too much arguments (>4)");
    }
}

pub fn push_args(args: &[Arg]) -> Instructions {
    args.iter()
        .enumerate()
        .fold(vec![], |mut acc: Instructions, (i, arg)| {
            acc.push(push_reg(i));
            match arg {
                Arg::Stack(loc) => {
                    acc.push(mov_rr(i, Register::Bp));
                    acc.push(sub_reg(i, *loc as i32));
                }
                Arg::Data(loc) => acc.push(mov_value(i, *loc)),
            }
            acc
        })
}

pub fn pop_args(args_count: usize) -> Instructions {
    (0..args_count)
        .enumerate()
        .fold(vec![], |mut acc: Instructions, (i, arg)| {
            acc.push(pop_reg(i));
            acc
        })
}
