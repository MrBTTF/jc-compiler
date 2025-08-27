use std::{
    collections::{BTreeMap, HashMap},
    mem, usize,
};

use crate::emitter::symbols::{Relocation, Symbol};

use super::{
    super::variables::DataRef,
    mnemonics::{self, Mnemonic, MnemonicName, SIZE_OF_JMP},
};

#[derive(Debug, Clone)]
pub struct Call {
    pub symbol: String,
    pub offsets: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CodeContext {
    instructions: Vec<Mnemonic>,
    pc: usize,
    offsets: Vec<usize>,
    calls: BTreeMap<String, Call>,
    labels: BTreeMap<String, usize>,
    symbols: BTreeMap<String, Symbol>,
    relocations: Vec<Relocation>,
    image_base: u64,
}

impl CodeContext {
    pub fn new(image_base: u64) -> Self {
        Self {
            instructions: vec![],
            pc: 0,
            offsets: vec![0],
            calls: BTreeMap::new(),
            labels: BTreeMap::new(),
            symbols: BTreeMap::new(),
            relocations: vec![],
            image_base,
        }
    }

    pub fn add(&mut self, mnemonic: Mnemonic) -> &mut Self {
        let code_size = self.get_code_size();
        self.instructions.push(mnemonic.clone());
        self.pc += 1;
        self.offsets
            .push(self.offsets.last().unwrap() + mnemonic.clone().as_vec().len());
        if let Some(symbol) = mnemonic.get_symbol() {
            let relocation = Relocation::new(
                symbol.name.to_string(),
                code_size + self.last().get_value_loc(),
                symbol._type,
            );
            self.relocations.push(relocation);
        }
        self
    }

    pub fn add_slice(&mut self, mnemonics: &[Mnemonic]) {
        for m in mnemonics.iter() {
            self.add(m.clone());
        }
    }

    pub fn set_label(&mut self, label: String) {
        self.labels.insert(label, self.get_code_size());
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

    pub fn get_calls(&self) -> BTreeMap<String, Call> {
        self.calls.clone()
    }

    pub fn get_relocations(&self) -> &[Relocation] {
        &self.relocations
    }

    pub fn get_label_offset(&self, label: &str) -> usize {
        *self.labels.get(label).unwrap()
    }

    pub fn get_labels(&self) -> BTreeMap<String, usize> {
        self.labels.clone()
    }

    pub fn compute_calls(
        &mut self,
        text_section_start: u32,
        external_symbols: HashMap<String, u32>,
    ) {
        for (name, call) in self.calls.clone().iter() {
            for c in call.offsets.iter() {
                assert_eq!(
                    self.instructions[*c].get_name(),
                    MnemonicName::Call,
                    "line: {}\n{}",
                    *c,
                    self.instructions[*c]
                );
                let call_address = self.get_code_size() - self.get_offset(*c + 1);

                self.get_mut(*c)
                    .set_op1(mnemonics::Operand::Offset32(call_address as i32));
            }

            self.add(mnemonics::JMP.op1(0_u32));
            let mut jump_offset = external_symbols[name];
            jump_offset -= text_section_start + self.get_code_size() as u32;
            self.instructions.last_mut().unwrap().set_op1(jump_offset);
        }
    }

    pub fn compute_data(&mut self, data_section_start: u32, const_data: BTreeMap<usize, DataRef>) {
        let mut data_cursor = 0;
        for (line, data_ref) in const_data.iter() {
            let address = self.image_base + data_section_start as u64 + data_cursor as u64;
            // println!("address: {:0x}", address);
            self.get_mut(*line).set_op2(address);
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

mod tests {
    use std::{
        io::{Read, Write},
        process::Command,
    };
    use tempfile::NamedTempFile;

    use crate::emitter::text::mnemonics::Operand::Offset32;
    use crate::emitter::text::{mnemonics::*, CodeContext};

    use rstest::*;

    fn compile_asm(code: &str) -> Vec<u8> {
        let code = format!("[bits 64]\n{code}");

        let mut src = NamedTempFile::new().unwrap();
        src.write_all(code.as_bytes()).unwrap();

        let mut bin = NamedTempFile::new().unwrap();
        let output = Command::new("nasm")
            .args([
                "-f",
                "bin",
                "-a",
                &src.path().as_os_str().to_str().unwrap(),
                "-o",
                &bin.path().to_str().unwrap(),
            ])
            .output()
            .unwrap();

        if !output.stderr.is_empty() {
            panic!(
                "{}\n{}",
                String::from_utf8(output.stderr).unwrap(),
                String::from_utf8(output.stdout).unwrap()
            );
        }

        let mut buf = vec![];
        bin.read_to_end(&mut buf).unwrap();
        buf.to_vec()
    }

    fn assert_eq_hex(actual: Vec<u8>, expected: Vec<u8>) {
        let hex_actual = actual
            .iter()
            .map(|b| format!("{b:0x}"))
            .collect::<Vec<String>>()
            .join(",");
        let hex_expected = expected
            .iter()
            .map(|b| format!("{b:0x}"))
            .collect::<Vec<String>>()
            .join(",");

        assert_eq!(actual, expected, "[{hex_actual}] != [{hex_expected}]");
    }

    #[rstest]
    fn test_mov() {
        let mut code = CodeContext::new(0);
        code.add_slice(&[
            MOV.op1(register::CX).op2(0xABCD_u16),
            MOV.op1(register::ECX).op2(0xABCDEF12_u32),
            MOV.op1(register::RCX).op2(0xABCDEF12ABCDEF12_u64),
            MOV.op1(register::CX).op2(register::AX),
            MOV.op1(register::ECX).op2(register::EAX),
            MOV.op1(register::RCX).op2(register::RAX),
        ]);
        let expected = compile_asm(
            "
             mov cx, 0xABCD
             mov ecx, 0xABCDEF12
             mov rcx, 0xABCDEF12ABCDEF12

             mov cx, ax
             mov ecx, eax
             mov rcx, rax
        ",
        );
        assert_eq_hex(code.to_bin(), expected);
    }

    #[rstest]
    fn test_loop() {
        let mut code = CodeContext::new(0);
        code.add_slice(&[
            XOR.op1(register::RCX).op2(register::RCX),
            INC.op1(register::RCX),
            CMP.op1(register::RCX).op2(0x32000_u32),
            JL.op1(Operand::Offset32(-0x13)),
        ]);
        let expected = compile_asm(
            "
         loop1:      xor rcx,rcx   ; cx-register is the counter, set to 0
                inc rcx          ; Increment
               cmp rcx, 0x32000    ; Compare cx to the limit
               jl 0x0   ; Loop while less or equal
        ",
        );
        assert_eq_hex(code.to_bin(), expected);
    }

    #[rstest]
    fn test_loop_with_stack() {
        let mut code = CodeContext::new(0);
        code.add_slice(&[
            INC.op1(register::RCX).disp(Offset32(0xFFFF)),
            CMP.op1(register::RCX)
                .op2(0x32000_u32)
                .disp(Offset32(0xFFFF)),
            JL.op1(Operand::Offset32(-0x18)),
        ]);
        let expected = compile_asm(
            "
         loop1:    inc qword [rcx+0xFFFF]
                   cmp qword [rcx+0xFFFF], 0x32000
                   jl 0x0   ; Loop while less or equal
        ",
        );
        assert_eq_hex(code.to_bin(), expected);
    }

    // #[rstest]
    // fn test_jmp() {
    //     let mut code = CodeContext::new(0);
    //     code.add_slice(&[JMP.op1(0xABCDEF_u32)]);
    //     let expected = compile_asm(
    //         "
    //          JMP DWORD [0xABCDEF]
    //     ",
    //     );
    //     assert_eq_hex(code.to_bin(), expected);
    // }
}
