use crate::ir::types::{BlockId, Instruction, Operation, TIRBlock, Terminator, VirtualRegister};

pub struct IRBuilder {
    pub blocks: Vec<TIRBlock>,
    current: BlockId,

    next_value: usize,
    next_block: usize,
}

impl IRBuilder {
    pub fn new() -> Self {
        let entry = BlockId(0);

        Self {
            blocks: vec![TIRBlock {
                label: entry,
                params: vec![],
                instructions: vec![],
                terminator: Terminator::Void,
            }],
            current: entry,
            next_value: 0,
            next_block: 1,
        }
    }

    pub fn value(&mut self) -> VirtualRegister {
        let value = VirtualRegister(self.next_value);
        self.next_value += 1;
        value
    }

    pub fn emit(&mut self, op: Operation) -> VirtualRegister {
        let dest = self.value();
        let instr = Instruction { dest, op };

        self.current_mut().instructions.push(instr);
        dest
    }

    pub fn terminate(&mut self, terminator: Terminator) {
        self.current_mut().terminator = terminator;
    }

    pub fn init_block(&mut self) -> BlockId {
        let block = BlockId(self.next_block);
        self.next_block += 1;
        self.blocks.push(TIRBlock {
            label: block,
            params: vec![],
            instructions: vec![],
            terminator: Terminator::Void,
        });
        block
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    pub fn to_blocks(self) -> Vec<TIRBlock> {
        self.blocks
    }

    pub fn current(&self) -> BlockId {
        self.current
    }

    pub fn current_mut(&mut self) -> &mut TIRBlock {
        self.blocks
            .get_mut(self.current.0)
            .expect("current block must exist")
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
