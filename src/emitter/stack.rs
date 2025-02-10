use crate::emitter::text::mnemonics::*;

#[derive(Debug, Clone)]
pub struct StackManager {}

impl StackManager {
    pub fn new() -> Self {
        StackManager {}
    }

    pub fn reset_stack(&mut self) -> Vec<Mnemonic> {
        vec![
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]
    }
}
