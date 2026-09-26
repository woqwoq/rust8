use crate::constants::{
    FONT, FONT_START_ADDRESS, KEYPAD_SIZE, RAM_SIZE, REGISTER_COUNT, ROM_START_ADDRESS,
    SCREEN_HEIGHT, SCREEN_WIDTH, STACK_SIZE,
};

pub struct Processor {
    pc: u16,                                           // Program counter
    stack: [u16; STACK_SIZE], // Stack for 16 16bit addressess for subroutines/functions
    stack_pointer: u8,        // 8bit stack pointer
    registers: [u8; REGISTER_COUNT], // 16 8bit registers: V0 - VF
    memory: [u8; RAM_SIZE],   // 4096B of RAM
    memory_pointer: u16,      // 16bit index register used as a memory pointer
    delay_timer: u8,          // 8bit Delay timer
    sound_timer: u8,          // 8bit Sound timer
    pub display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // 64 * 32 Bool Display
    pub keypad: [bool; KEYPAD_SIZE], // 16 value 1bit Keypad
    pub is_dirty: bool,       // Decides whether or not to update the display
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    UnknownInstruction,
    RegisterIndexOutOfRange(usize),
    MemoryIndexOutOfRange(usize),
    StackOverflow(usize),
    StackUnderflow(usize),
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Instruction {
    CLS,                               // 00E0 - Clear screen
    RET,                               // 00EE - Return from subroutine
    JP(u16),                           // 1NNN - Jump to address NNN
    CALL(u16),                         // 2NNN - Call subroutine at address NNN
    SE { x: usize, nn: u8 },           // 3XNN - Skip instruction if Vx == NN
    SNE { x: usize, nn: u8 },          // 4XNN - Skip instruction if Vx != NN
    SRE { x: usize, y: usize },        // 5XY0 - Skip instruction if Vx == Vy
    LD { x: usize, nn: u8 },           // 6XNN - Set register Vx = NN
    ADD { x: usize, nn: u8 },          // 7XNN - Add NN to register Vx (Vx += NN)
    LDR { x: usize, y: usize },        // 8XY0 - Set register Vx = Vy
    OR { x: usize, y: usize },         // 8XY1 - Set Vx = Vx OR Vy
    AND { x: usize, y: usize },        // 8XY2 - Set Vx = Vx AND Vy
    XOR { x: usize, y: usize },        // 8XY3 - Set Vx = Vx XOR Vy
    ADDR { x: usize, y: usize },       // 8XY4 - Set Vx = Vx + Vy, set Vf = Carry
    SUBR { x: usize, y: usize },       // 8XY5 - Set Vx = Vx - Vy, set Vf = Not Borrow
    LDI(u16),                          // ANNN - Set memory pointer I = NNN
    DRW { x: usize, y: usize, n: u8 }, // DXYN - Draw n-byte sprite at (Vx, Vy)
    Undefined(u16),
}

impl Processor {
    fn load_font(memory: &mut [u8; RAM_SIZE], starting_addr: u16, font: &[u8; 80]) {
        for (offset, val) in font.iter().enumerate() {
            memory[starting_addr as usize + offset] = *val;
        }
    }

    pub fn new() -> Self {
        let mut memory = [0; RAM_SIZE];
        Self::load_font(&mut memory, FONT_START_ADDRESS, &FONT);

        Processor {
            pc: ROM_START_ADDRESS, // We initialize PC at 512, as that's where user programs live
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            registers: [0; REGISTER_COUNT],
            memory,
            memory_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keypad: [false; KEYPAD_SIZE],
            is_dirty: false,
        }
    }

    fn load_in_ram(&mut self, starting_addr: u16, data: &[u8]) -> Result<(), RuntimeError> {
        for (offset, item) in data.iter().enumerate() {
            let memory_index = starting_addr as usize + offset;
            if memory_index >= self.memory.len() {
                return Err(RuntimeError::MemoryIndexOutOfRange(memory_index));
            }
            self.memory[starting_addr as usize + offset] = *item;
        }
        Ok(())
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), RuntimeError> {
        self.load_in_ram(ROM_START_ADDRESS, data)
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn advance_pc(&mut self) {
        self.pc += 2;
    }

    fn get_register(&self, x: usize) -> Result<&u8, RuntimeError> {
        if x >= REGISTER_COUNT {
            return Err(RuntimeError::RegisterIndexOutOfRange(x));
        }

        Ok(&self.registers[x])
    }

    fn get_register_mut(&mut self, x: usize) -> Result<&mut u8, RuntimeError> {
        if x >= REGISTER_COUNT {
            return Err(RuntimeError::RegisterIndexOutOfRange(x));
        }

        Ok(&mut self.registers[x])
    }

    fn push_to_stack(&mut self, x: u16) -> Result<(), RuntimeError> {
        if self.stack_pointer as usize >= STACK_SIZE {
            return Err(RuntimeError::StackOverflow(self.pc as usize));
        }

        self.stack[self.stack_pointer as usize] = x;
        self.stack_pointer += 1;

        Ok(())
    }

    fn pop_from_stack(&mut self) -> Result<u16, RuntimeError> {
        if self.stack_pointer == 0 {
            return Err(RuntimeError::StackUnderflow(self.pc as usize));
        }

        self.stack_pointer -= 1;
        Ok(self.stack[self.stack_pointer as usize])
    }

    fn read_memory_range(
        &mut self,
        start_address: u16,
        end_address: u16,
    ) -> Result<&[u8], RuntimeError> {
        Ok(&self.memory[(start_address as usize)..(end_address as usize)])
    }

    fn read_memory_byte(&mut self, address: u16) -> Result<u8, RuntimeError> {
        Ok(self.memory[address as usize])
    }

    fn get_display_mut(&mut self, x: usize, y: usize) -> Option<&mut bool> {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return None;
        }

        Some(&mut self.display[x + y * SCREEN_WIDTH])
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
            (0x0, 0x0, 0xE, 0x0) => Instruction::CLS,
            (0x0, 0x0, 0xE, 0xE) => Instruction::RET,
            (0x1, _, _, _) => Instruction::JP(nnn),
            (0x2, _, _, _) => Instruction::CALL(nnn),
            (0x3, x, _, _) => Instruction::SE { x: x as usize, nn },
            (0x4, x, _, _) => Instruction::SNE { x: x as usize, nn },
            (0x5, x, y, 0x0) => Instruction::SRE {
                x: x as usize,
                y: y as usize,
            },
            (0x6, x, _, _) => Instruction::LD { x: x as usize, nn },
            (0x7, x, _, _) => Instruction::ADD { x: x as usize, nn },
            (0x8, x, y, 0x0) => Instruction::LDR {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x1) => Instruction::OR {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x2) => Instruction::AND {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x3) => Instruction::XOR {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x4) => Instruction::ADDR {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x5) => Instruction::SUBR {
                x: x as usize,
                y: y as usize,
            },
            (0xA, _, _, _) => Instruction::LDI(nnn),
            (0xD, x, y, _) => Instruction::DRW {
                x: x as usize,
                y: y as usize,
                n,
            },
            _ => Instruction::Undefined(instruction),
        }
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::CLS => self.instruction_cls(), // 00E0 - Clear screen
            Instruction::RET => self.instruction_ret(), // 00EE - Return from subroutine
            Instruction::JP(nnn) => self.instruction_jp(nnn), // 1NNN - Jump to address NNN
            Instruction::CALL(nnn) => self.instruction_call(nnn), // 2NNN - Call subroutine at address NNN
            Instruction::SE { x, nn } => self.instruction_se(x, nn), // 3XNN - Skip instruction if Vx == NN
            Instruction::SNE { x, nn } => self.instruction_sne(x, nn), // 4XNN - Skip instruction if Vx != NN
            Instruction::SRE { x, y } => self.instruction_sre(x, y), // 5XY0 - Skip instruction if Vx == Vy
            Instruction::LD { x, nn } => self.instruction_ld(x, nn), // 6XNN - Set register Vx = NN
            Instruction::ADD { x, nn } => self.instruction_add(x, nn), // 7XNN - Add NN to register Vx (Vx += NN)
            Instruction::LDR { x, y } => self.instruction_ldr(x, y),
            Instruction::OR { x, y } => self.instruction_or(x, y),
            Instruction::AND { x, y } => self.instruction_and(x, y),
            Instruction::XOR { x, y } => self.instruction_xor(x, y),
            Instruction::ADDR { x, y } => self.instruction_addr(x, y),
            Instruction::SUBR { x, y } => self.instruction_subr(x, y),
            Instruction::LDI(nnn) => self.instruction_ldi(nnn), // ANNN - Set memory pointer I = NNN
            Instruction::DRW { x, y, n } => self.instruction_drw(x, y, n), // DXYN - Draw n-byte sprite at (Vx, Vy)
            _ => Err(RuntimeError::UnknownInstruction),
        }
    }

    pub fn instruction_cls(&mut self) -> Result<(), RuntimeError> {
        self.display.fill(false);
        Ok(())
    }

    pub fn instruction_ret(&mut self) -> Result<(), RuntimeError> {
        self.pc = self.pop_from_stack()?;
        Ok(())
    }

    pub fn instruction_jp(&mut self, nnn: u16) -> Result<(), RuntimeError> {
        self.pc = nnn;
        Ok(())
    }

    pub fn instruction_call(&mut self, nnn: u16) -> Result<(), RuntimeError> {
        self.push_to_stack(self.pc)?;
        self.pc = nnn;
        Ok(())
    }

    pub fn instruction_se(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        if *self.get_register(x)? == nn {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_sne(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        if *self.get_register(x)? != nn {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_sre(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        if *self.get_register(x)? == *self.get_register(y)? {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_ld(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        *self.get_register_mut(x)? = nn;

        Ok(())
    }

    pub fn instruction_add(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        *self.get_register_mut(x)? = self.get_register(x)?.wrapping_add(nn);

        Ok(())
    }

    pub fn instruction_ldr(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        *self.get_register_mut(x)? = *self.get_register(y)?;

        Ok(())
    }

    pub fn instruction_or(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = *self.get_register(x)?;
        let vy = *self.get_register(y)?;

        *self.get_register_mut(x)? = vx | vy;

        Ok(())
    }

    pub fn instruction_and(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = *self.get_register(x)?;
        let vy = *self.get_register(y)?;

        *self.get_register_mut(x)? = vx & vy;

        Ok(())
    }

    pub fn instruction_xor(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = *self.get_register(x)?;
        let vy = *self.get_register(y)?;

        *self.get_register_mut(x)? = vx ^ vy;

        Ok(())
    }

    pub fn instruction_addr(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = *self.get_register(x)?;
        let vy = *self.get_register(y)?;

        let (res, ovf) = vx.overflowing_add(vy);

        *self.get_register_mut(x)? = res;
        *self.get_register_mut(0xF)? = ovf as u8;

        Ok(())
    }

    pub fn instruction_subr(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = *self.get_register(x)?;
        let vy = *self.get_register(y)?;

        let (res, ovf) = vx.overflowing_sub(vy);

        *self.get_register_mut(x)? = res;
        *self.get_register_mut(0xF)? = !ovf as u8;

        Ok(())
    }

    pub fn instruction_ldi(&mut self, nnn: u16) -> Result<(), RuntimeError> {
        self.memory_pointer = nnn;

        Ok(())
    }

    pub fn instruction_drw(&mut self, x: usize, y: usize, n: u8) -> Result<(), RuntimeError> {
        *self.get_register_mut(0xF)? = 0; // Reset collision register

        // Wrap for initial coordinates
        let vx = *self.get_register(x)? as usize % SCREEN_WIDTH;
        let vy = *self.get_register(y)? as usize % SCREEN_HEIGHT;

        for row in 0..n {
            // Load first byte (row) of a sprite (e.g **** | 11110000 | 0xF0)
            let sprite_byte = self.read_memory_byte(self.memory_pointer + (row as u16))?;

            for col in 0..8 {
                // If we are inside of the range of the current screen, we render XOR of current
                // sprite bit and current displayed bit
                if let Some(display_bit) = self.get_display_mut(vx + col, vy + row as usize) {
                    // Extract col-th bit from our sprite's byte (row)
                    let bit = (sprite_byte >> (7 - col as u8) & 1) == 1;

                    // If current display bit changes to 0, we mark a collision
                    let collision = *display_bit && bit;

                    // current display bit XOR current sprite bit
                    *display_bit ^= bit;

                    if collision {
                        *self.get_register_mut(0xF)? = 1;
                    }
                }
            }
        }

        self.is_dirty = true;

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), RuntimeError> {
        let opcode = self.fetch();
        let instruction = Self::decode(opcode);
        self.execute(instruction)
    }
}

#[cfg(test)]
mod processor_tests {
    use crate::{
        constants::{SCREEN_HEIGHT, SCREEN_WIDTH},
        processor::{FONT_START_ADDRESS, Instruction, Processor, ROM_START_ADDRESS, RuntimeError},
    };

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
        assert_eq!(Processor::decode(0x00E0), Instruction::CLS);
        assert_eq!(Processor::decode(0x1123), Instruction::JP(0x123));
        assert_eq!(
            Processor::decode(0xD12F),
            Instruction::DRW {
                x: 0x1,
                y: 0x2,
                n: 0xF
            }
        );
        assert_eq!(
            Processor::decode(0x6F12),
            Instruction::LD { x: 0xF, nn: 0x12 }
        );
    }

    #[test]
    fn get_register() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(matches!(proc.get_register(1), Ok(0)));
        assert!(matches!(proc.get_register_mut(15), Ok(0)));

        assert!(matches!(
            proc.get_register_mut(16),
            Err(RuntimeError::RegisterIndexOutOfRange(16))
        ));
    }

    #[test]
    fn instruction_jp() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        assert_eq!(proc.pc, ROM_START_ADDRESS);
        for i in ROM_START_ADDRESS..=ROM_START_ADDRESS + 5 {
            assert!(proc.instruction_jp(i).is_ok());
            assert_eq!(proc.pc, i);
        }
    }

    #[test]
    fn instruction_ld() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        for i in 0..15 {
            assert_eq!(*proc.get_register(i).unwrap(), 0);
            assert!(proc.instruction_ld(i, i as u8).is_ok());
            assert_eq!(*proc.get_register(i).unwrap(), i as u8);
        }
    }

    #[test]
    fn instruction_add() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        for i in 0..15 {
            assert_eq!(*proc.get_register(i).unwrap(), 0);
            assert!(proc.instruction_add(i, i as u8).is_ok());
            assert_eq!(*proc.get_register(i).unwrap(), i as u8);
        }

        assert!(proc.instruction_add(1, 13).is_ok());
        assert_eq!(*proc.get_register(1).unwrap(), 14);

        assert!(proc.instruction_add(0, 255).is_ok());
        assert!(proc.instruction_add(0, 1).is_ok());
        assert_eq!(*proc.get_register(0).unwrap(), 0);
    }

    #[test]
    fn instruction_drw() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        proc.memory_pointer = FONT_START_ADDRESS;

        assert!(proc.instruction_drw(0, 0, 5).is_ok());

        assert!(proc.display[0]);
        assert!(proc.display[1]);
        assert!(proc.display[2]);
        assert!(proc.display[3]);

        assert!(proc.display.iter().filter(|p| **p).count() == 14);
    }

    #[test]
    fn instruction_drw_double_draw_removes_sprite() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        proc.memory_pointer = FONT_START_ADDRESS;

        assert!(proc.instruction_drw(0, 0, 5).is_ok());
        assert!(proc.display.iter().filter(|p| **p).count() != 0);
        assert_eq!(*proc.get_register(0xF).unwrap(), 0);

        assert!(proc.instruction_drw(0, 0, 5).is_ok());
        assert!(proc.display.iter().filter(|p| **p).count() == 0);
        assert_eq!(*proc.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn instruction_drw_wraps_and_gets_clipped_when_out_of_range() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        proc.memory_pointer = FONT_START_ADDRESS;

        // Should wrap to (63, 31) because of modulo
        *proc.get_register_mut(0x0).unwrap() = (SCREEN_WIDTH + SCREEN_WIDTH - 1) as u8;
        *proc.get_register_mut(0x1).unwrap() = (SCREEN_HEIGHT + SCREEN_HEIGHT - 1) as u8;

        assert!(proc.instruction_drw(0x0, 0x1, 5).is_ok());

        // Only a single bottom right pixel should get drawn
        assert!(proc.display.iter().filter(|p| **p).count() == 1);
    }

    #[test]
    fn stack_push_ok() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 0..16 {
            assert!(proc.push_to_stack(i).is_ok());
        }

        for i in (0..16).rev() {
            println!("{:?}", proc.stack);
            assert_eq!(proc.pop_from_stack().unwrap(), i);
        }
    }

    #[test]
    fn stack_push_err_stack_overflow() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 0..16 {
            assert!(proc.push_to_stack(i).is_ok());
        }

        assert!(matches!(
            proc.push_to_stack(16),
            Err(RuntimeError::StackOverflow(512))
        ));
    }

    #[test]
    fn stack_push_err_stack_underflow() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(matches!(
            proc.pop_from_stack(),
            Err(RuntimeError::StackUnderflow(512))
        ));
    }

    #[test]
    fn instruction_call_ok() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        // 512 -> 560 -> 580
        assert!(proc.instruction_call(560).is_ok());
        assert!(proc.instruction_call(580).is_ok());
        assert!(proc.stack_pointer == 2);
        assert_eq!(proc.pop_from_stack().unwrap(), 560);
        assert_eq!(proc.pop_from_stack().unwrap(), 512);
    }

    #[test]
    fn instruction_call_err_ovf() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 512..528 {
            assert!(proc.instruction_call(i).is_ok());
        }
        assert!(matches!(
            proc.instruction_call(528),
            Err(RuntimeError::StackOverflow(527))
        ));
    }

    #[test]
    fn instruction_ret_ok() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        // 512 -> 560 -> 580
        assert!(proc.instruction_call(560).is_ok());
        assert!(proc.instruction_call(580).is_ok());

        assert!(proc.instruction_ret().is_ok());
        assert_eq!(proc.pc, 560);
        assert!(proc.instruction_ret().is_ok());
        assert_eq!(proc.pc, 512);
    }

    #[test]
    fn instruction_ret_err_udf() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(matches!(
            proc.instruction_ret(),
            Err(RuntimeError::StackUnderflow(512))
        ));
    }

    #[test]
    fn instruction_se() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x5).unwrap() = 0xEF;

        assert!(proc.instruction_se(0x5, 0xFF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        assert!(proc.instruction_se(0x5, 0xEF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_sne() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x5).unwrap() = 0xEF;

        assert!(proc.instruction_sne(0x5, 0xEF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        assert!(proc.instruction_sne(0x5, 0xFF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_sre() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x5).unwrap() = 0xEF;
        *proc.get_register_mut(0xF).unwrap() = 0xEF;

        assert!(proc.instruction_sre(0x5, 0xE).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        assert!(proc.instruction_sre(0x5, 0xF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_ldr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0xF).unwrap() = 0xEF;

        assert!(proc.instruction_ldr(0x0, 0xF).is_ok());
        assert_eq!(
            *proc.get_register(0x0).unwrap(),
            *proc.get_register(0xF).unwrap()
        );
    }

    #[test]
    fn instruction_or() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b10101010;
        *proc.get_register_mut(0xF).unwrap() = 0b01010101;

        assert!(proc.instruction_or(0x0, 0xF).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0b11111111);
    }

    #[test]
    fn instruction_and() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b10101010;
        *proc.get_register_mut(0xF).unwrap() = 0b00001111;

        assert!(proc.instruction_and(0x0, 0xF).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0b00001010);
    }

    #[test]
    fn instruction_xor() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b00001111;
        *proc.get_register_mut(0xF).unwrap() = 0b11111111;

        assert!(proc.instruction_xor(0x0, 0xF).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0b11110000);
    }

    #[test]
    fn instruction_addr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0x01;
        *proc.get_register_mut(0xF).unwrap() = 0x0E;

        *proc.get_register_mut(0xE).unwrap() = 0xF1;

        assert!(proc.instruction_addr(0x0, 0xF).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0xF);
        assert_eq!(*proc.get_register(0xF).unwrap(), 0);

        assert!(proc.instruction_addr(0x0, 0xE).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0);
        assert_eq!(*proc.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn instruction_subr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xFF;
        *proc.get_register_mut(0xF).unwrap() = 0xF0;

        *proc.get_register_mut(0xE).unwrap() = 0x10;

        assert!(proc.instruction_subr(0x0, 0xF).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 0xF);
        assert_eq!(*proc.get_register(0xF).unwrap(), 1);

        assert!(proc.instruction_subr(0x0, 0xE).is_ok());
        assert_eq!(*proc.get_register(0x0).unwrap(), 255);
        assert_eq!(*proc.get_register(0xF).unwrap(), 0);
    }
}
