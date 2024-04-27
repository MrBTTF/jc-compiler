use std::{collections::BTreeMap, mem, usize};

use super::mnemonics::Mnemonic;

#[derive(Debug, Clone)]
pub struct CodeContext {
    instructions: Vec<Mnemonic>,
    pc: usize,
    offsets: Vec<usize>,
    calls: BTreeMap<usize, String>,
}

impl CodeContext {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            pc: 0,
            offsets: vec![0],
            calls: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, mnemonic: Mnemonic) {
        if mnemonic.get_name() == "CALL" {
            self.calls
                .insert(self.pc, mnemonic.get_symbol().unwrap().to_string());
        }
        self.instructions.push(mnemonic.clone());
        self.pc += 1;
        self.offsets
            .push(self.offsets.last().unwrap() + mnemonic.clone().as_vec().len());
    }

    pub fn add_slice(&mut self, mnemonics: &[Mnemonic]) {
        for m in mnemonics.iter() {
            self.add(m.clone());
        }
    }

    pub fn get_pc(&self) -> usize {
        self.pc
    }

    pub fn get(&self, i: usize) -> Mnemonic {
        self.instructions[i].clone()
    }

    pub fn get_mut(&mut self, i: usize) -> &mut Mnemonic {
        self.instructions.get_mut(i).unwrap()
    }

    pub fn get_offset(&self, i: usize) -> usize {
        self.offsets[i]
    }

    pub fn last(&self) -> &Mnemonic {
        self.instructions.last().unwrap()
    }

    pub fn get_calls(&self) -> &BTreeMap<usize, String> {
        &self.calls
    }

    pub fn to_bin(&self) -> Vec<u8> {
        self.instructions.iter().fold(vec![], |mut acc, instr| {
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
