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

    pub fn init_function_stack(&mut self, regs: &[register::Register]) -> Vec<Mnemonic> {
        self.function_tops.push(0);

        let mut code = self.push_register(register::RBP);
        self.shrink_function_stack(8);
        code.push(MOV.op1(register::RBP).op2(register::RSP));
        code.extend(self.push_registers(&regs));
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

    pub fn free_function_stack(&mut self, regs: &[register::Register]) -> Vec<Mnemonic> {
        let mut code = self.pop_registers(&regs);
        code.push(POP.op1(register::RBP));
        self.function_tops.pop();
        code
    }

    pub fn block_stack_size(&self) -> usize {
        self.function_stack_size() - *self.block_bottoms.last().unwrap()
    }

    pub fn function_stack_size(&self) -> usize {
        *self.function_tops.last().unwrap()
    }

    pub fn grow_function_stack(&mut self, v: usize) {
        let top = self.function_tops.last_mut().unwrap();
        *top += v;
    }

    pub fn shrink_function_stack(&mut self, v: usize) {
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
        self.push_registers(&[reg])
    }

    pub fn push_registers(&mut self, regs: &[register::Register]) -> Vec<Mnemonic> {
        if regs.is_empty() {
            return vec![];
        }
        self.grow_function_stack(regs.len() * 8);
        regs.iter()
            .map(|&reg| PUSH.op1(reg))
            .collect::<Vec<Mnemonic>>()
    }

    pub fn pop_register(&mut self, reg: register::Register) -> Vec<Mnemonic> {
        self.pop_registers(&[reg])
    }

    pub fn pop_registers(&mut self, regs: &[register::Register]) -> Vec<Mnemonic> {
        if regs.is_empty() {
            return vec![];
        }
        self.shrink_function_stack(regs.len() * 8);
        regs.iter()
            .map(|&reg| POP.op1(reg))
            .collect::<Vec<Mnemonic>>()
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
