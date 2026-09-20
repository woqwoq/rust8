pub struct Processor {
    pc: u16,                  // Program counter
    stack: [u16; 16],         // Stack for 16 16bit addressess for subroutines/functions
    stack_pointer: u8,        // 8bit stack pointer
    registers: [u8; 16],      // 16 8bit registers: V0 - VF
    memory: [u8; 4096],       // 4096B of RAM
    memory_pointer: u16,      // 16bit index register used as a memory pointer
    delay_timer: u8,          // 8bit Delay timer
    sound_timer: u8,          // 8bit Sound timer
    display: [bool; 64 * 32], // 64 * 32 Bool Display
    keypad: [bool; 16],       // 16 value 1bit Keypad
}

pub enum RuntimeError {
    General,
}

pub enum Instruction {
    ClearScreen,                          // 00E0
    Jump(u16),                            // 1NNN
    SetRegister { x: u8, nn: u8 },        // 6XNN
    AddValueToRegister { x: u8, nn: u8 }, // 7XNN
    SetMemoryPointer(u16),                // ANNN
    Draw { x: u8, y: u8, n: u8 },         // DXYN
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            pc: 512,
            stack: [0; 16],
            stack_pointer: 0,
            registers: [0; 16],
            memory: [0; 4096],
            memory_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
        }
    }

    pub fn fetch(&mut self) -> u16 {
        todo!()
    }

    pub fn decode(&self, instruction: u16) -> Instruction {
        todo!()
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        todo!()
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }
}
