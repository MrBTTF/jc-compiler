use std::vec;

use crate::emitter::text::mnemonics::*;

#[derive(Debug, Clone)]
pub struct StackManager {
    block_bottoms: Vec<usize>,
    function_tops: Vec<usize>,
    was_aligned: bool,
}

impl StackManager {
    pub fn new() -> Self {
        StackManager {
            block_bottoms: vec![],
            function_tops: vec![0],
            was_aligned: false,
        }
    }

    pub fn init_function_stack(&mut self) -> Vec<Mnemonic> {
        self.function_tops.push(0);
        // when function body is entered, the return address is pushed to stack implicitly
        self.grow_function_stack(8);

        let mut code = self.push_register(register::RBP);
        code.push(MOV.op1(register::RBP).op2(register::RSP));
        code
    }

    pub fn init_stack(&mut self) {
        self.block_bottoms.push(self.function_stack_size());
    }

    pub fn free(&mut self) -> Vec<Mnemonic> {
        let code = if self.block_stack_size() > 0 {
            let local_size = self.block_stack_size();
            self.shrink_function_stack(local_size);
            vec![ADD.op1(register::RSP).op2(local_size as u32)]
        } else {
            vec![]
        };

        self.block_bottoms.pop();
        code
    }

    pub fn free_function_stack(&mut self) -> Vec<Mnemonic> {
        self.shrink_function_stack(8); // return address
        self.function_tops.pop();
        vec![POP.op1(register::RBP)]
    }

    pub fn block_stack_size(&self) -> usize {
        self.function_stack_size() - *self.block_bottoms.last().unwrap()
    }

    fn function_stack_size(&self) -> usize {
        *self.function_tops.last().unwrap()
    }

    fn grow_function_stack(&mut self, v: usize) {
        let top = self.function_tops.last_mut().unwrap();
        *top += v;
    }

    fn shrink_function_stack(&mut self, v: usize) {
        let top = self.function_tops.last_mut().unwrap();
        *top -= v;
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
        self.grow_function_stack(8);
        vec![MOV.op1(register::RAX).op2(data), PUSH.op1(register::RAX)]
    }

    pub fn push_register(&mut self, reg: register::Register) -> Vec<Mnemonic> {
        self.grow_function_stack(8);
        vec![PUSH.op1(reg)]
    }

    pub fn pop_register(&mut self, reg: register::Register) -> Vec<Mnemonic> {
        self.shrink_function_stack(8);
        vec![POP.op1(reg)]
    }

    pub fn align_for_call(&mut self) -> Vec<Mnemonic> {
        if self.function_stack_size() % 16 != 0 {
            self.was_aligned = true;
            self.grow_function_stack(8);
            vec![SUB.op1(register::RSP).op2(8_u32)]
        } else {
            vec![]
        }
    }

    pub fn unalign_after_call(&mut self) -> Vec<Mnemonic> {
        // after call the alignment is always % 16 so we need to know if we did alignment before
        // or the stack had been already aligned before the call
        if self.was_aligned {
            self.was_aligned = false;
            self.shrink_function_stack(8);
            vec![ADD.op1(register::RSP).op2(8_u32)]
        } else {
            vec![]
        }
    }
}
