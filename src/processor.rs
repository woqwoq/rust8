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
const ROM_START_ADDRESS: u16 = 0x200; // 512

pub struct Processor {
    pc: u16,                      // Program counter
    stack: [u16; 16],             // Stack for 16 16bit addressess for subroutines/functions
    stack_pointer: u8,            // 8bit stack pointer
    registers: [u8; 16],          // 16 8bit registers: V0 - VF
    memory: [u8; 4096],           // 4096B of RAM
    memory_pointer: u16,          // 16bit index register used as a memory pointer
    delay_timer: u8,              // 8bit Delay timer
    sound_timer: u8,              // 8bit Sound timer
    pub display: [bool; 64 * 32], // 64 * 32 Bool Display
    keypad: [bool; 16],           // 16 value 1bit Keypad
    pub is_dirty: bool,           // Decides whether or not to update the display
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    General,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    ClearScreen,                             // 00E0
    Jump(u16),                               // 1NNN
    SetRegister { x: usize, nn: u8 },        // 6XNN
    AddValueToRegister { x: usize, nn: u8 }, // 7XNN
    SetMemoryPointer(u16),                   // ANNN
    Draw { x: usize, y: usize, n: u8 },      // DXYN
    Undefined(u16),
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
            pc: ROM_START_ADDRESS, // We initialize PC at 512, as that's where user programs live
            stack: [0; 16],
            stack_pointer: 0,
            registers: [0; 16],
            memory,
            memory_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
            is_dirty: false,
        }
    }

    pub fn advance_pc(&mut self) {
        self.pc += 2;
    }

    pub fn fetch(&mut self) -> u16 {
        let upper = self.memory[(self.pc) as usize];
        let lower = self.memory[(self.pc + 1) as usize];

        self.advance_pc();

        (upper as u16) << 8 | lower as u16
    }

    pub fn decode(instruction: u16) -> Instruction {
        let n = (instruction & 0x000F) as u8;
        let nn = (instruction & 0x00FF) as u8;
        let nnn = instruction & 0x0FFF;

        let nibbles = (
            ((instruction >> 12) & 0xF) as u8,
            ((instruction >> 8) & 0xF) as u8,
            ((instruction >> 4) & 0xF) as u8,
            (instruction & 0xF) as u8,
        );

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => Instruction::ClearScreen,
            (0x1, _, _, _) => Instruction::Jump(nnn),
            (0x6, x, _, _) => Instruction::SetRegister { x: x as usize, nn },
            (0x7, x, _, _) => Instruction::AddValueToRegister { x: x as usize, nn },
            (0xA, _, _, _) => Instruction::SetMemoryPointer(nnn),
            (0xD, x, y, _) => Instruction::Draw {
                x: x as usize,
                y: y as usize,
                n,
            },
            _ => Instruction::Undefined(instruction),
        }
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        todo!()
    }

    pub fn step(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[cfg(test)]
mod processor_tests {
    use crate::processor::{Instruction, Processor, ROM_START_ADDRESS};

    fn load_in_ram(memory: &mut [u8; 4096], starting_addr: u16, data: Vec<u8>) {
        for (offset, item) in data.iter().enumerate() {
            memory[starting_addr as usize + offset] = *item;
        }
    }

    fn inspect_ram_bytes(memory: &mut [u8; 4096], starting_addr: u16, ending_addr: u16) -> &[u8] {
        &memory[starting_addr as usize..ending_addr as usize]
    }

    fn create_processor_with_ram_filled(data: Vec<u8>) -> Processor {
        let mut proc = Processor::new();
        load_in_ram(&mut proc.memory, ROM_START_ADDRESS, data.clone());

        assert_eq!(
            inspect_ram_bytes(
                &mut proc.memory,
                ROM_START_ADDRESS,
                ROM_START_ADDRESS + data.len() as u16
            ),
            &data
        );

        proc
    }

    #[test]
    fn fetch_works1() {
        let program = vec![0x00, 0xE0, 0x6F, 0x12];
        let mut proc = create_processor_with_ram_filled(program.clone());
        assert_eq!(proc.fetch(), 0x00E0);
        assert_eq!(proc.fetch(), 0x6F12);
    }

    #[test]
    fn fetch_works2() {
        let program = vec![];
        let mut proc = create_processor_with_ram_filled(program.clone());

        assert_eq!(proc.fetch(), 0x0000);
    }

    #[test]
    fn decode_works() {
        assert_eq!(Processor::decode(0x00E0), Instruction::ClearScreen);
        assert_eq!(Processor::decode(0x1123), Instruction::Jump(0x123));
        assert_eq!(
            Processor::decode(0xD12F),
            Instruction::Draw {
                x: 0x1,
                y: 0x2,
                n: 0xF
            }
        );
        assert_eq!(
            Processor::decode(0x6F12),
            Instruction::SetRegister { x: 0xF, nn: 0x12 }
        );
    }
}
