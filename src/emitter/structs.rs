use std::{
    collections::{BTreeMap, HashMap},
    mem, usize,
};

use super::{
    data::DataRef,
    exe::sections::IMAGE_BASE,
    mnemonics::{self, Mnemonic, SIZE_OF_JMP},
};

#[derive(Debug, Clone)]
pub struct CodeContext {
    instructions: Vec<Mnemonic>,
    pc: usize,
    offsets: Vec<usize>,
    calls: BTreeMap<String, Vec<usize>>,
    const_data: BTreeMap<usize, DataRef>,
}

impl CodeContext {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            pc: 0,
            offsets: vec![0],
            calls: BTreeMap::new(),
            const_data: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, mnemonic: Mnemonic) -> &mut Self {
        if mnemonic.get_name() == "CALL" {
            let symbol = mnemonic.get_symbol().unwrap().to_string();
            self.calls.entry(symbol).or_default().push(self.pc);
        }
        self.instructions.push(mnemonic.clone());
        self.pc += 1;
        self.offsets
            .push(self.offsets.last().unwrap() + mnemonic.clone().as_vec().len());
        self
    }

    pub fn with_const_data(&mut self, data: Vec<u8>) {
        self.const_data.insert(
            self.get_pc() - 1,
            DataRef {
                offset: self.last().get_value_loc(),
                data,
            },
        );
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

    pub fn get_code_size(&self) -> usize {
        *self.offsets.last().unwrap()
    }

    pub fn get_code_size_with_calls(&self) -> usize {
        self.get_code_size() + *SIZE_OF_JMP * self.calls.len()
    }

    pub fn last(&self) -> &Mnemonic {
        self.instructions.last().unwrap()
    }

    pub fn get_calls(&self) -> BTreeMap<String, Vec<usize>> {
        self.calls.clone()
    }

    pub fn get_const_data(&self) -> BTreeMap<usize, DataRef> {
        self.const_data.clone()
    }

    pub fn compute_calls(
        &mut self,
        text_section_start: u32,
        external_symbols: HashMap<String, u32>,
    ) {
        for (call, locs) in self.calls.clone().iter() {
            for c in locs.iter() {
                assert!(
                    self.instructions[*c].get_name() == "CALL",
                    "line: {}\n{}",
                    *c,
                    self.instructions[*c]
                );
                let call_address = self.get_code_size() - self.get_offset(*c + 1);

                self.get_mut(*c)
                    .set_op1(mnemonics::Operand::Offset32(call_address as u32));
            }

            self.add(mnemonics::JMP.op1(mnemonics::Operand::Imm32(0)));
            let mut jump_offset = external_symbols[call];
            jump_offset -= text_section_start + self.get_code_size() as u32;
            self.instructions
                .last_mut()
                .unwrap()
                .set_op1(mnemonics::Operand::Imm32(jump_offset));
        }
    }

    pub fn compute_data(&mut self, data_section_start: u32, const_data: BTreeMap<usize, DataRef>) {
        let mut data_cursor = 0;
        for (line, data_ref) in const_data.iter() {
            let address = IMAGE_BASE + data_section_start as u64 + data_cursor as u64;
            // println!("address: {:0x}", address);
            self.get_mut(*line)
                .set_op2(mnemonics::Operand::Imm64(address));
            data_cursor += data_ref.data.len();
        }
    }

    pub fn to_bin(&self) -> Vec<u8> {
        self.instructions.iter().fold(vec![], |mut acc, instr| {
            acc.extend(instr.to_owned().as_vec());
            acc
        })
    }
}

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
