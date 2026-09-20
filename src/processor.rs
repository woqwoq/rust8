const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const FONT_START_ADDRESS: u16 = 0x50;

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
    fn load_font(memory: &mut [u8; 4096], starting_addr: u16, font: &[u8; 80]) {
        for (offset, val) in font.iter().enumerate() {
            memory[starting_addr as usize + offset] = *val;
        }
    }

    pub fn new() -> Self {
        let mut memory = [0; 4096];
        Self::load_font(&mut memory, FONT_START_ADDRESS, &FONT);

        Processor {
            pc: 512,
            stack: [0; 16],
            stack_pointer: 0,
            registers: [0; 16],
            memory,
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
