use std::mem;

use super::{
    amd64::*,
    elf::structs::{ELFHeader, ProgramHeader, VIRTUAL_ADDRESS_START},
};

pub fn print(data_loc: u32, length: usize) -> Vec<u8> {
    let entry_point: u32 = (mem::size_of::<ELFHeader>() + mem::size_of::<ProgramHeader>() * 3)
        .try_into()
        .unwrap();

    let data_loc = VIRTUAL_ADDRESS_START + entry_point + data_loc;
    [
        Mov32::build(Register::Ecx, data_loc as i32),
        Mov32::build(Register::Edx, length as i32),
        Mov32::build(Register::Ebx, STDOUT_FD),
        Mov32::build(Register::Eax, SYS_WRITE),
        SysCall::build(),
    ]
    .concat()
}

pub fn print_let(data_loc: u32, length: usize) -> Vec<u8> {
    let length = length + 4 - (length % 4);
    let data_loc = (data_loc as i32) + (length as i32);
    [
        Mov32rr::build(Register::Ecx, Register::Ebp),
        Sub32::build(Register::Ecx, data_loc),
        Mov32::build(Register::Edx, length as i32),
        Mov32::build(Register::Ebx, STDOUT_FD),
        Mov32::build(Register::Eax, SYS_WRITE),
        SysCall::build(),
    ]
    .concat()
}
