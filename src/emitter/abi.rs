use super::{amd64::*, structs::Instructions};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

const ARG_REGISTERS: &[Register] = &[Register::Ax];

pub fn push_args(args: &[Arg]) -> Instructions {
    args.iter()
        .enumerate()
        .fold(vec![], |mut acc: Instructions, (i, arg)| {
            let arg_reg = ARG_REGISTERS[i];
            acc.push(Push::new(arg_reg));
            match arg {
                Arg::Stack(loc) => {
                    acc.push(Mov64rr::new(arg_reg, Register::Bp));
                    acc.push(Sub64::new(arg_reg, *loc as i32));
                }
                Arg::Data(loc) => acc.push(Mov64::new(arg_reg, *loc)),
            }
            acc
        })
}

pub fn pop_args(args_count: usize) -> Instructions {
    ARG_REGISTERS[..args_count]
        .iter()
        .fold(vec![], |mut acc, arg_reg| {
            acc.push(Pop::new(*arg_reg));
            acc
        })
}
