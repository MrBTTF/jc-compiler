use super::amd64::*;

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

const ARG_REGISTERS: &[Register] = &[Register::Ax];

pub fn push_args(args: &[Arg]) -> Vec<u8> {
    args.iter().enumerate().fold(vec![], |mut acc, (i, arg)| {
        let arg_reg = ARG_REGISTERS[i];
        acc.extend(Push::build(arg_reg));
        match arg {
            Arg::Stack(loc) => {
                acc.extend(Mov64rr::build(arg_reg, Register::Bp));
                acc.extend(Sub64::build(arg_reg, *loc as i32));
            }
            Arg::Data(loc) => acc.extend(Mov64::build(arg_reg, *loc)),
        }
        acc
    })
}

pub fn pop_args(args_count: usize) -> Vec<u8> {
    ARG_REGISTERS[..args_count]
        .iter()
        .fold(vec![], |mut acc, arg_reg| {
            acc.extend(Pop::build(*arg_reg));
            acc
        })
}
