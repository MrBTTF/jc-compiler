use std::mem;

use super::mnemonics::Mnemonic;

pub type Instructions = Vec<Mnemonic>;

pub trait InstructionsTrait {
    fn to_bin(&self) -> Vec<u8>;
}

impl InstructionsTrait for Instructions {
    fn to_bin(&self) -> Vec<u8> {
        self.iter().fold(vec![], |mut acc, instr| {
            acc.extend(instr.to_owned().as_vec());
            acc
        })
    }
}

// pub trait Instruction: std::fmt::Debug {
//     fn as_slice(&self) -> &[u8] {
//         let data_ptr = self as *const _ as *const u8;
//         let size = mem::size_of_val(self);
//         unsafe { std::slice::from_raw_parts(data_ptr, size) }
//     }

//     fn as_vec(&self) -> Vec<u8> {
//         self.as_slice().to_vec()
//     }
// }

// trait InstructionClone {
//     fn clone_dyn(&self) -> Box<dyn InstructionClone>;
// }

// impl InstructionClone for Box<dyn Instruction> {
//     fn clone_dyn(&self) -> Box<dyn InstructionClone> {
//         *self.clone()
//     }
// }

// impl Clone for Box<dyn Instruction> {
//     fn clone(&self) -> Self {
//         Box::new(*self.clone())
//     }
// }

pub trait Sliceable: Sized {
    fn as_slice(&self) -> &[u8] {
        let data_ptr = self as *const _ as *const u8;
        let size = mem::size_of::<Self>();
        unsafe { std::slice::from_raw_parts(data_ptr, size) }
    }

    fn as_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}
