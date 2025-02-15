use std::vec;

use crate::emitter::text::mnemonics::*;

#[derive(Debug, Clone)]
pub struct StackManager {
    pub tops: Vec<usize>,
    pub size: usize,
    pub aligned: bool,
}

impl StackManager {
    pub fn new() -> Self {
        StackManager {
            tops: vec![],
            size: 0,
            aligned: false,
        }
    }

    pub fn init_function_stack(&mut self) -> Vec<Mnemonic> {
        self.grow(8); //  RBP
        self.tops.push(0);
        self.grow(8); //  RBP
        vec![
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]
    }

    pub fn reset_stack(&mut self) -> Vec<Mnemonic> {
        self.tops.push(0);
        self.grow(8);
        vec![
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]
    }

    pub fn drop(&mut self) -> Vec<Mnemonic> {
        self.shrink(8);
        let mut code = if self.get_top() > 0 {
            vec![ADD.op1(register::RSP).op2(self.get_top() as u32)]
        } else {
            vec![]
        };

        self.tops.pop();
        code.push(POP.op1(register::RBP));
        code
    }

    pub fn drop_function_stack(&mut self) -> Vec<Mnemonic> {
        let code = self.drop();
        self.shrink(8);
        code
    }

    pub fn get_top(&self) -> usize {
        *self.tops.last().unwrap()
    }

    fn grow(&mut self, v: usize) {
        let last = self.tops.last_mut().unwrap();
        *last += v;
        self.size += v;
    }

    fn shrink(&mut self, v: usize) {
        let last = self.tops.last_mut().unwrap();
        *last -= v;
        self.size -= v;
    }

    pub fn push_list(&mut self, data: &[u64], size: usize) -> Vec<Mnemonic> {
        if data.len() == 0 {
            return vec![];
        }

        let push_list = data.iter().fold(vec![], |mut acc: Vec<Mnemonic>, value| {
            acc.extend(self.push(*value));
            acc
        });

        let push_length = self.push(size as u64);
        [push_list, push_length].concat()
    }

    pub fn push(&mut self, data: u64) -> Vec<Mnemonic> {
        self.grow(8);
        vec![MOV.op1(register::RAX).op2(data), PUSH.op1(register::RAX)]
    }

    pub fn push_register(&mut self, reg: register::Register) -> Vec<Mnemonic> {
        self.grow(8);
        vec![PUSH.op1(reg)]
    }

    pub fn pop(&mut self) -> Vec<Mnemonic> {
        self.shrink(8);
        todo!()
    }

    pub fn pop_register(&mut self, reg: register::Register) -> Vec<Mnemonic> {
        self.shrink(8);
        vec![POP.op1(reg)]
    }

    pub fn align_for_call(&mut self) -> Vec<Mnemonic> {
        if self.size % 16 != 0 {
            self.aligned = true;
            self.grow(8);
            vec![SUB.op1(register::RSP).op2(8_u32)]
        } else {
            vec![]
        }
    }

    pub fn align_local(&mut self) -> Vec<Mnemonic> {
        if self.get_top() % 16 == 0 {
            self.grow(8);
            vec![SUB.op1(register::RSP).op2(8_u32)]
        } else {
            vec![]
        }
    }

    pub fn unalign_after_call(&mut self) -> Vec<Mnemonic> {
        // after call the alignment is always % 16 so we need to know if did alignment before
        // or the stack had been already aligned before the call
        if self.aligned {
            self.aligned = false;
            self.shrink(8);
            vec![ADD.op1(register::RSP).op2(8_u32)]
        } else {
            vec![]
        }
    }

    pub fn free(&mut self) -> Vec<Mnemonic> {
        let code = if self.get_top() > 0 {
            vec![
                POP.op1(register::RBP),
                ADD.op1(register::RSP).op2(self.get_top() as u32),
            ]
        } else {
            vec![]
        };

        self.tops.pop();
        code
    }
}
