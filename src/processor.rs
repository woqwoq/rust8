use crate::constants::{
    FONT, FONT_START_ADDRESS, KEYPAD_SIZE, RAM_SIZE, REGISTER_COUNT, ROM_START_ADDRESS,
    SCREEN_HEIGHT, SCREEN_WIDTH, SPRITE_SIZE, STACK_SIZE,
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
    KeypadIndexOutOfRange(usize),
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
    SHR { x: usize, y: usize },        // 8XY6 - Set Vx = Vx SHR 1, set Vf = LSB == 1
    SUBRN { x: usize, y: usize },      // 8XY7 - Set Vx = Vy - Vx, set Vf = Not Borrow
    SHL { x: usize, y: usize },        // 8XYE - Set Vx = Vx SHL 1, set Vf = LSB == 1
    SNER { x: usize, y: usize },       // 9XY0 - Skip instruction if Vx != Vy
    LDI(u16),                          // ANNN - Set memory pointer I = NNN
    JPR(u16),                          // BNNN - Jump to address NNN + V0
    RND { x: usize, nn: u8 },          // CXNN - Set Vx = random byte AND NN
    DRW { x: usize, y: usize, n: u8 }, // DXYN - Draw n-byte sprite at (Vx, Vy)
    SKP(usize),                        // EX9E - Skip instruction if key in Vx is pressed
    SKNP(usize),                       // EXA1 - Skip instruction if key in Vx is not pressed
    LDDT(usize),                       // FX07 - Set Vx = DT value
    LDK(usize),                        // FX0A - Wait for a key, store the value of the key in Vx
    LDDTR(usize),                      // FX15 - Set delay timer = Vx
    LDST(usize),                       // FX18 - Set sound timer = Vx
    ADDI(usize),                       // FX1E - Set I = I + Vx
    LDS(usize),                        // FX29 - Set I = location of sprite for digit in Vx
    LDB(usize),                        // FX33 - Store digits of Vx in memory at I, I+1, and I+2
    LDRM(usize),                       // FX55 - Store V0 - Vx in memory at I
    LDMR(usize),                       // FX65 - Read V0 - Vx from memory at I
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
            *self.get_memory_byte_mut(memory_index)? = *item;
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

    pub fn rollback_pc(&mut self) {
        self.pc -= 2;
    }

    fn get_register(&self, x: usize) -> Result<u8, RuntimeError> {
        if x >= REGISTER_COUNT {
            return Err(RuntimeError::RegisterIndexOutOfRange(x));
        }

        Ok(self.registers[x])
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

    fn is_key_pressed(&self, key: u8) -> Result<bool, RuntimeError> {
        if key as usize >= KEYPAD_SIZE {
            return Err(RuntimeError::KeypadIndexOutOfRange(key as usize));
        }
        Ok(self.keypad[key as usize])
    }

    fn get_memory_byte(&self, address: usize) -> Result<u8, RuntimeError> {
        if address >= RAM_SIZE {
            return Err(RuntimeError::MemoryIndexOutOfRange(address));
        }

        Ok(self.memory[address])
    }

    fn get_memory_byte_mut(&mut self, address: usize) -> Result<&mut u8, RuntimeError> {
        if address >= RAM_SIZE {
            return Err(RuntimeError::MemoryIndexOutOfRange(address));
        }

        Ok(&mut self.memory[address])
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
            (0x8, x, y, 0x6) => Instruction::SHR {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0x7) => Instruction::SUBRN {
                x: x as usize,
                y: y as usize,
            },
            (0x8, x, y, 0xE) => Instruction::SHL {
                x: x as usize,
                y: y as usize,
            },
            (0x9, x, y, 0x0) => Instruction::SNER {
                x: x as usize,
                y: y as usize,
            },
            (0xA, _, _, _) => Instruction::LDI(nnn),
            (0xB, _, _, _) => Instruction::JPR(nnn),
            (0xC, x, _, _) => Instruction::RND { x: x as usize, nn },
            (0xD, x, y, _) => Instruction::DRW {
                x: x as usize,
                y: y as usize,
                n,
            },
            (0xE, x, 0x9, 0xE) => Instruction::SKP(x as usize),
            (0xE, x, 0xA, 0x1) => Instruction::SKNP(x as usize),
            (0xF, x, 0x0, 0x7) => Instruction::LDDT(x as usize),
            (0xF, x, 0x0, 0xA) => Instruction::LDK(x as usize),
            (0xF, x, 0x1, 0x5) => Instruction::LDDTR(x as usize),
            (0xF, x, 0x1, 0x8) => Instruction::LDST(x as usize),
            (0xF, x, 0x1, 0xE) => Instruction::ADDI(x as usize),
            (0xF, x, 0x2, 0x9) => Instruction::LDS(x as usize),
            (0xF, x, 0x3, 0x3) => Instruction::LDB(x as usize),
            (0xF, x, 0x5, 0x5) => Instruction::LDRM(x as usize),
            (0xF, x, 0x6, 0x5) => Instruction::LDMR(x as usize),
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
            Instruction::LDR { x, y } => self.instruction_ldr(x, y), // 8XY0 - Set register Vx = Vy
            Instruction::OR { x, y } => self.instruction_or(x, y),   // 8XY1 - Set Vx = Vx OR Vy
            Instruction::AND { x, y } => self.instruction_and(x, y), // 8XY2 - Set Vx = Vx AND Vy
            Instruction::XOR { x, y } => self.instruction_xor(x, y), // 8XY3 - Set Vx = Vx XOR Vy
            Instruction::ADDR { x, y } => self.instruction_addr(x, y), // 8XY4 - Set Vx = Vx + Vy, set Vf = Carry
            Instruction::SUBR { x, y } => self.instruction_subr(x, y), // 8XY5 - Set Vx = Vx - Vy, set Vf = Not Borrow
            Instruction::SHR { x, .. } => self.instruction_shr(x), // 8XY6 - Set Vx = Vx SHR 1, set Vf = LSB == 1
            Instruction::SUBRN { x, y } => self.instruction_subrn(x, y), // 8XY7 - Set Vx = Vy - Vx, set Vf = Not Borrow
            Instruction::SHL { x, .. } => self.instruction_shl(x), // 8XYE - Set Vx = Vx SHL 1, set Vf = LSB == 1
            Instruction::SNER { x, y } => self.instruction_sner(x, y), // 9XY0 - Skip instruction if Vx != Vy
            Instruction::LDI(nnn) => self.instruction_ldi(nnn), // ANNN - Set memory pointer I = NNN
            Instruction::JPR(nnn) => self.instruction_jpr(nnn), // BNNN - Jump to address NNN + V0
            Instruction::RND { x, nn } => self.instruction_rnd(x, nn),
            Instruction::DRW { x, y, n } => self.instruction_drw(x, y, n), // DXYN - Draw n-byte sprite at (Vx, Vy)
            Instruction::SKP(x) => self.instruction_skp(x),
            Instruction::SKNP(x) => self.instruction_sknp(x),
            Instruction::LDDT(x) => self.instruction_lddt(x),
            Instruction::LDK(x) => self.instruction_ldk(x),
            Instruction::LDDTR(x) => self.instruction_lddtr(x),
            Instruction::LDST(x) => self.instruction_ldst(x),
            Instruction::ADDI(x) => self.instruction_addi(x),
            Instruction::LDS(x) => self.instruction_lds(x),
            Instruction::LDB(x) => self.instruction_ldb(x),
            Instruction::LDRM(x) => self.instruction_ldrm(x),
            Instruction::LDMR(x) => self.instruction_ldmr(x),
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
        if self.get_register(x)? == nn {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_sne(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        if self.get_register(x)? != nn {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_sre(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        if self.get_register(x)? == self.get_register(y)? {
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
        *self.get_register_mut(x)? = self.get_register(y)?;

        Ok(())
    }

    pub fn instruction_or(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        *self.get_register_mut(x)? = vx | vy;

        Ok(())
    }

    pub fn instruction_and(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        *self.get_register_mut(x)? = vx & vy;

        Ok(())
    }

    pub fn instruction_xor(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        *self.get_register_mut(x)? = vx ^ vy;

        Ok(())
    }

    pub fn instruction_addr(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        let (res, ovf) = vx.overflowing_add(vy);

        *self.get_register_mut(x)? = res;
        *self.get_register_mut(0xF)? = ovf as u8;

        Ok(())
    }

    pub fn instruction_subr(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        let (res, ovf) = vx.overflowing_sub(vy);

        *self.get_register_mut(x)? = res;
        *self.get_register_mut(0xF)? = !ovf as u8;

        Ok(())
    }

    pub fn instruction_shr(&mut self, x: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;

        *self.get_register_mut(x)? = vx >> 1;
        *self.get_register_mut(0xF)? = ((vx & 0x1) == 1) as u8; // Vf = LSB(Vx) == 1

        Ok(())
    }

    pub fn instruction_subrn(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let vy = self.get_register(y)?;

        let (res, ovf) = vy.overflowing_sub(vx);

        *self.get_register_mut(x)? = res;
        *self.get_register_mut(0xF)? = !ovf as u8;

        Ok(())
    }

    pub fn instruction_shl(&mut self, x: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;

        *self.get_register_mut(x)? = vx << 1;
        *self.get_register_mut(0xF)? = ((vx >> 7) == 1) as u8; // Vf = MSB(Vx) == 1

        Ok(())
    }

    pub fn instruction_sner(&mut self, x: usize, y: usize) -> Result<(), RuntimeError> {
        if self.get_register(x)? != self.get_register(y)? {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_ldi(&mut self, nnn: u16) -> Result<(), RuntimeError> {
        self.memory_pointer = nnn;

        Ok(())
    }

    pub fn instruction_jpr(&mut self, nnn: u16) -> Result<(), RuntimeError> {
        self.pc = self.get_register(0x0)? as u16 + nnn;

        Ok(())
    }

    pub fn instruction_rnd(&mut self, x: usize, nn: u8) -> Result<(), RuntimeError> {
        *self.get_register_mut(x)? = rand::random::<u8>() & nn;

        Ok(())
    }

    pub fn instruction_drw(&mut self, x: usize, y: usize, n: u8) -> Result<(), RuntimeError> {
        *self.get_register_mut(0xF)? = 0; // Reset collision register

        // Wrap for initial coordinates
        let vx = self.get_register(x)? as usize % SCREEN_WIDTH;
        let vy = self.get_register(y)? as usize % SCREEN_HEIGHT;

        for row in 0..n as usize {
            // Load first byte (row) of a sprite (e.g **** | 11110000 | 0xF0)
            let sprite_byte = self.get_memory_byte(self.memory_pointer as usize + row)?;

            for col in 0..8 {
                // If we are inside of the range of the current screen, we render XOR of current
                // sprite bit and current displayed bit
                if let Some(display_bit) = self.get_display_mut(vx + col, vy + row) {
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

    pub fn instruction_skp(&mut self, x: usize) -> Result<(), RuntimeError> {
        if self.is_key_pressed(self.get_register(x)?)? {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_sknp(&mut self, x: usize) -> Result<(), RuntimeError> {
        if !self.is_key_pressed(self.get_register(x)?)? {
            self.advance_pc();
        }

        Ok(())
    }

    pub fn instruction_lddt(&mut self, x: usize) -> Result<(), RuntimeError> {
        *self.get_register_mut(x)? = self.delay_timer;

        Ok(())
    }

    pub fn instruction_ldk(&mut self, x: usize) -> Result<(), RuntimeError> {
        let _ = self.get_register(x)?; // fail-fast when register is out of range

        if let Some(v) = self.keypad.iter().position(|&pressed| pressed) {
            *self.get_register_mut(x)? = v as u8;
        } else {
            self.rollback_pc();
        }

        Ok(())
    }

    pub fn instruction_lddtr(&mut self, x: usize) -> Result<(), RuntimeError> {
        self.delay_timer = self.get_register(x)?;

        Ok(())
    }

    pub fn instruction_ldst(&mut self, x: usize) -> Result<(), RuntimeError> {
        self.sound_timer = self.get_register(x)?;

        Ok(())
    }

    pub fn instruction_addi(&mut self, x: usize) -> Result<(), RuntimeError> {
        self.memory_pointer += self.get_register(x)? as u16;

        Ok(())
    }

    pub fn instruction_lds(&mut self, x: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)? as u16 & 0x0F;

        self.memory_pointer = FONT_START_ADDRESS + (SPRITE_SIZE * vx);

        Ok(())
    }

    pub fn instruction_ldb(&mut self, x: usize) -> Result<(), RuntimeError> {
        let vx = self.get_register(x)?;
        let i = self.memory_pointer as usize;

        *self.get_memory_byte_mut(i)? = vx / 100;
        *self.get_memory_byte_mut(i + 1)? = (vx % 100) / 10;
        *self.get_memory_byte_mut(i + 2)? = vx % 10;

        Ok(())
    }

    pub fn instruction_ldrm(&mut self, x: usize) -> Result<(), RuntimeError> {
        for i in 0..=x {
            *self.get_memory_byte_mut(i + self.memory_pointer as usize)? = self.get_register(i)?;
        }

        Ok(())
    }

    pub fn instruction_ldmr(&mut self, x: usize) -> Result<(), RuntimeError> {
        for i in 0..=x {
            *self.get_register_mut(i)? = self.get_memory_byte(i + self.memory_pointer as usize)?;
        }

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
        constants::{FONT, RAM_SIZE, REGISTER_COUNT, SCREEN_HEIGHT, SCREEN_WIDTH, SPRITE_SIZE},
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
    fn get_memory_byte() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 0..100 {
            *proc.get_memory_byte_mut(i).unwrap() = i as u8;
        }

        for i in 0..100 {
            assert_eq!(proc.get_memory_byte(i).unwrap(), i as u8);
        }

        assert!(matches!(
            proc.get_memory_byte(RAM_SIZE + 1),
            Err(RuntimeError::MemoryIndexOutOfRange(_))
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
            assert_eq!(proc.get_register(i).unwrap(), 0);
            assert!(proc.instruction_ld(i, i as u8).is_ok());
            assert_eq!(proc.get_register(i).unwrap(), i as u8);
        }
    }

    #[test]
    fn instruction_add() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        for i in 0..15 {
            assert_eq!(proc.get_register(i).unwrap(), 0);
            assert!(proc.instruction_add(i, i as u8).is_ok());
            assert_eq!(proc.get_register(i).unwrap(), i as u8);
        }

        assert!(proc.instruction_add(1, 13).is_ok());
        assert_eq!(proc.get_register(1).unwrap(), 14);

        assert!(proc.instruction_add(0, 255).is_ok());
        assert!(proc.instruction_add(0, 1).is_ok());
        assert_eq!(proc.get_register(0).unwrap(), 0);
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
        assert_eq!(proc.get_register(0xF).unwrap(), 0);

        assert!(proc.instruction_drw(0, 0, 5).is_ok());
        assert!(proc.display.iter().filter(|p| **p).count() == 0);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);
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
            proc.get_register(0x0).unwrap(),
            proc.get_register(0xF).unwrap()
        );
    }

    #[test]
    fn instruction_or() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b10101010;
        *proc.get_register_mut(0xF).unwrap() = 0b01010101;

        assert!(proc.instruction_or(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0b11111111);
    }

    #[test]
    fn instruction_and() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b10101010;
        *proc.get_register_mut(0xF).unwrap() = 0b00001111;

        assert!(proc.instruction_and(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0b00001010);
    }

    #[test]
    fn instruction_xor() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0b00001111;
        *proc.get_register_mut(0xF).unwrap() = 0b11111111;

        assert!(proc.instruction_xor(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0b11110000);
    }

    #[test]
    fn instruction_addr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0x01;
        *proc.get_register_mut(0xF).unwrap() = 0x0E;

        *proc.get_register_mut(0xE).unwrap() = 0xF1;

        assert!(proc.instruction_addr(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0xF);
        assert_eq!(proc.get_register(0xF).unwrap(), 0);

        assert!(proc.instruction_addr(0x0, 0xE).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn instruction_subr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xFF;
        *proc.get_register_mut(0xF).unwrap() = 0xF0;

        *proc.get_register_mut(0xE).unwrap() = 0x10;

        assert!(proc.instruction_subr(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0xF);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);

        assert!(proc.instruction_subr(0x0, 0xE).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 255);
        assert_eq!(proc.get_register(0xF).unwrap(), 0);
    }

    #[test]
    fn instruction_shr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(proc.instruction_shr(0x0).is_ok());
        assert_eq!(proc.get_register(0xF).unwrap(), 0);

        *proc.get_register_mut(0x0).unwrap() = 0b00001111;
        assert!(proc.instruction_shr(0x0).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0b000000111);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn instruction_subrn() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xF0;
        *proc.get_register_mut(0xF).unwrap() = 0xFF;

        *proc.get_register_mut(0xE).unwrap() = 0xE;

        assert!(proc.instruction_subrn(0x0, 0xF).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0xF);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);

        assert!(proc.instruction_subrn(0x0, 0xE).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 255);
        assert_eq!(proc.get_register(0xF).unwrap(), 0);
    }

    #[test]
    fn instruction_shl() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(proc.instruction_shl(0x0).is_ok());
        assert_eq!(proc.get_register(0xF).unwrap(), 0);

        *proc.get_register_mut(0x0).unwrap() = 0b1000_1110;
        assert!(proc.instruction_shl(0x0).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0b0001_1100);
        assert_eq!(proc.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn instruction_sner() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0xE).unwrap() = 0xFF;
        *proc.get_register_mut(0xF).unwrap() = 0xFF;

        assert!(proc.instruction_sner(0xE, 0xF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        assert!(proc.instruction_sner(0x0, 0xF).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_jpr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xF;

        assert!(proc.instruction_jpr(0x20F).is_ok());
        assert_eq!(proc.pc, 0x20F + 0xF);
    }

    #[test]
    fn instruction_rnd() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xFF;
        assert!(proc.instruction_rnd(0x0, 0x00).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 0);

        assert!(proc.instruction_rnd(0x0, 0xF0).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap() & 0x0F, 0);

        assert!(proc.instruction_rnd(0x0, 0x0F).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap() & 0xF0, 0);

        assert!(matches!(
            proc.instruction_rnd(16, 0xFF),
            Err(RuntimeError::RegisterIndexOutOfRange(16))
        ));
    }

    #[test]
    fn instruction_skp() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xF;

        assert!(proc.instruction_skp(0x0).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        proc.keypad[0xF] = true;
        assert!(proc.instruction_skp(0x0).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_sknp() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0xF;

        proc.keypad[0xF] = true;
        assert!(proc.instruction_sknp(0x0).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        proc.keypad[0xF] = false;
        assert!(proc.instruction_sknp(0x0).is_ok());
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_lddt() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        proc.delay_timer = 49;
        assert_eq!(proc.get_register(0x0).unwrap(), 0);

        assert!(proc.instruction_lddt(0x0).is_ok());
        assert_eq!(proc.get_register(0x0).unwrap(), 49);
    }

    #[test]
    fn is_key_pressed() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        assert!(matches!(proc.is_key_pressed(0x0), Ok(false)));
        assert!(matches!(proc.is_key_pressed(0xF), Ok(false)));

        proc.keypad[0x0] = true;
        proc.keypad[0xF] = true;
        assert!(matches!(proc.is_key_pressed(0x0), Ok(true)));
        assert!(matches!(proc.is_key_pressed(0xF), Ok(true)));

        assert!(matches!(
            proc.is_key_pressed(0xFF),
            Err(RuntimeError::KeypadIndexOutOfRange(0xFF))
        ));
    }

    #[test]
    fn instruction_ldk() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        assert!(matches!(
            proc.instruction_ldk(0xFF),
            Err(RuntimeError::RegisterIndexOutOfRange(0xFF))
        ));

        proc.pc += 2;
        assert!(proc.instruction_ldk(0xF).is_ok());
        assert_eq!(proc.get_register(0xF).unwrap(), 0);
        assert_eq!(proc.pc, ROM_START_ADDRESS);

        proc.pc += 2;
        proc.keypad[0xE] = true;
        assert!(proc.instruction_ldk(0xF).is_ok());
        assert_eq!(proc.get_register(0xF).unwrap(), 0xE);
        assert_eq!(proc.pc, ROM_START_ADDRESS + 2);
    }

    #[test]
    fn instruction_lddtr() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        *proc.get_register_mut(0x0).unwrap() = 0x80;

        assert_eq!(proc.delay_timer, 0);

        assert!(proc.instruction_lddtr(0x0).is_ok());
        assert_eq!(proc.delay_timer, 0x80);
    }

    #[test]
    fn instruction_ldst() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        *proc.get_register_mut(0x0).unwrap() = 0x80;

        assert_eq!(proc.sound_timer, 0);

        assert!(proc.instruction_ldst(0x0).is_ok());
        assert_eq!(proc.sound_timer, 0x80);
    }

    #[test]
    fn instruction_addi() {
        let mut proc = create_processor_with_ram_filled(vec![]);
        *proc.get_register_mut(0x0).unwrap() = 0x80;

        assert_eq!(proc.memory_pointer, 0);

        assert!(proc.instruction_addi(0x0).is_ok());
        assert_eq!(proc.memory_pointer, 0x80);
    }

    #[test]
    fn instruction_lds() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 0x80;

        assert_eq!(proc.memory_pointer, 0);

        for i in 0..FONT.len() / SPRITE_SIZE as usize {
            *proc.get_register_mut(0x0).unwrap() = i as u8;
            assert!(proc.instruction_lds(0x0).is_ok());

            for j in 0..SPRITE_SIZE {
                assert_eq!(
                    proc.get_memory_byte((proc.memory_pointer + j) as usize)
                        .unwrap(),
                    FONT[i * SPRITE_SIZE as usize + j as usize]
                );
            }
        }
    }

    #[test]
    fn instruction_ldb() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        *proc.get_register_mut(0x0).unwrap() = 123;

        assert_eq!(
            proc.get_memory_byte(proc.memory_pointer as usize).unwrap(),
            0
        );
        assert_eq!(
            proc.get_memory_byte(1 + proc.memory_pointer as usize)
                .unwrap(),
            0
        );
        assert_eq!(
            proc.get_memory_byte(2 + proc.memory_pointer as usize)
                .unwrap(),
            0
        );

        assert!(proc.instruction_ldb(0x0).is_ok());
        assert_eq!(
            proc.get_memory_byte(proc.memory_pointer as usize).unwrap(),
            1
        );
        assert_eq!(
            proc.get_memory_byte(1 + proc.memory_pointer as usize)
                .unwrap(),
            2
        );
        assert_eq!(
            proc.get_memory_byte(2 + proc.memory_pointer as usize)
                .unwrap(),
            3
        );

        *proc.get_register_mut(0x0).unwrap() = 250;
        assert!(proc.instruction_ldb(0x0).is_ok());
        assert_eq!(
            proc.get_memory_byte(proc.memory_pointer as usize).unwrap(),
            2
        );
        assert_eq!(
            proc.get_memory_byte(1 + proc.memory_pointer as usize)
                .unwrap(),
            5
        );
        assert_eq!(
            proc.get_memory_byte(2 + proc.memory_pointer as usize)
                .unwrap(),
            0
        );

        *proc.get_register_mut(0x0).unwrap() = 42;
        assert!(proc.instruction_ldb(0x0).is_ok());
        assert_eq!(
            proc.get_memory_byte(proc.memory_pointer as usize).unwrap(),
            0
        );
        assert_eq!(
            proc.get_memory_byte(1 + proc.memory_pointer as usize)
                .unwrap(),
            4
        );
        assert_eq!(
            proc.get_memory_byte(2 + proc.memory_pointer as usize)
                .unwrap(),
            2
        );

        *proc.get_register_mut(0x0).unwrap() = 0;
        assert!(proc.instruction_ldb(0x0).is_ok());
        assert_eq!(
            proc.get_memory_byte(proc.memory_pointer as usize).unwrap(),
            0
        );
        assert_eq!(
            proc.get_memory_byte(1 + proc.memory_pointer as usize)
                .unwrap(),
            0
        );
        assert_eq!(
            proc.get_memory_byte(2 + proc.memory_pointer as usize)
                .unwrap(),
            0
        );
    }

    #[test]
    fn instruction_ldrm() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 0..REGISTER_COUNT as u8 {
            *proc.get_register_mut(i as usize).unwrap() = i;
        }

        assert!(proc.instruction_ldrm(0xF).is_ok());

        for i in 0..REGISTER_COUNT as u8 {
            assert_eq!(
                proc.get_memory_byte((i as u16 + proc.memory_pointer) as usize)
                    .unwrap(),
                i
            );
        }
    }

    #[test]
    fn instruction_ldmr() {
        let mut proc = create_processor_with_ram_filled(vec![]);

        for i in 0..REGISTER_COUNT as u8 {
            *proc
                .get_memory_byte_mut((i as u16 + proc.memory_pointer) as usize)
                .unwrap() = i;
        }

        assert!(proc.instruction_ldmr(0xF).is_ok());

        for i in 0..REGISTER_COUNT as u8 {
            assert_eq!(*proc.get_register_mut(i as usize).unwrap(), i);
        }
    }
}
