use rand::random;

const MEM_SIZE: usize = 4096;
const V_REG_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;
const START_ADDR: u16 = 0x200; // start address for all chip 8 programs

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Chip8 {
    pc: u16,                                      // Program Counter
    ram: [u8; MEM_SIZE],                          // RAM
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // Display Screen
    v_reg: [u8; V_REG_SIZE],                      // V registers
    i_reg: u16,                                   // Indexing Register
    sp: u16,                                      // Stack pointer
    stack: [u16; STACK_SIZE],                     // CPU stack
    dt: u8,                                       // delay timer
    st: u8,                                       // sound timer
    keys: [bool; KEYPAD_SIZE],                    // Keypad
}

impl Chip8 {
    /// Chip 8 Initialization
    pub fn new() -> Self {
        let mut new_chip8 = Self {
            pc: START_ADDR,
            ram: [0; MEM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_reg: [0; V_REG_SIZE],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; KEYPAD_SIZE],
            dt: 0,
            st: 0,
        };

        // important gor fx29 instruction
        new_chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_chip8
    }

    /// Push u16 to stack
    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    /// Pop u16 from stack
    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
        // possible underflow - panics
    }

    /// Reset chip8
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; MEM_SIZE];
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
        self.v_reg = [0; V_REG_SIZE];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; KEYPAD_SIZE];
        self.dt = 0;
        self.st = 0;
    }

    pub fn tick(&mut self) {
        // 1. Get value specified at memory address stored in Program Counter
        let op = self.fetch();
        // 2. Decode this instruction
        // 3. Execute
        self.execute(op);
        // 4. Move program counter to next instruction set
    }

    fn fetch(&mut self) -> u16 {
        // 4 bytes representing the instruction
        // most significant and least significant represnests the op code
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = data.len() + START_ADDR as usize;
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        match (d1, d2, d3, d4) {
            (0, 0, 0, 0) => return,                                                // NOP
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH], // clear screen
            (0, 0, 0xE, 0xE) => {
                // RET
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            (1, _, _, _) => {
                //JMP NNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (2, _, _, _) => {
                // CALL addr
                let addr = op & 0xFFF;
                self.push(self.pc);
                self.pc = addr;
            }
            (3, _, _, _) => {
                // SKIP next if VX == NN
                // 3XNN

                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2
                }
            }
            (4, _, _, _) => {
                // Skip next if Vx != kk
                // 4XKK
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                // skip next instruction if Vx = Vy
                // 5xy0
                let x = d2 as usize;
                let y = d3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                // set Vx = kk
                // 6xkk
                let x = d2 as usize;
                let kk = (op & 0xFF) as u8;
                self.v_reg[x] = kk;
            }
            (7, _, _, _) => {
                // set Vx = Vx + kk
                // 7xkk
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            (8, _, _, 0) => {
                // set Vx = Vy
                // 8xy0
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            (8, _, _, 1) => {
                // set Vx = Vx or Vy
                // 8xy1
                self.v_reg[d2 as usize] |= self.v_reg[d3 as usize];
            }
            (8, _, _, 2) => {
                // set Vx = Vx and Vy
                // 8xy2
                self.v_reg[d2 as usize] &= self.v_reg[d3 as usize];
            }
            (8, _, _, 3) => {
                // set Vx = Vx xor Vy
                // 8xy3
                self.v_reg[d2 as usize] ^= self.v_reg[d3 as usize];
            }
            (8, _, _, 4) => {
                // sets Vx = Vx + Vy, set VF = carry
                // Values of Vx and Vy are added together.  If reult is greater than 8 bits, VF is set to 1, otherwise 0.  Lowest 8 bits are saved in Vx
                // 8xy4
                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 5) => {
                // Set Vx = Vx - Vy, set VF = NOT borrow
                // if Vx > Vy, then VF is set to 1, otherwise 0.  Then Vy is subtracted from Vx, result is stored in Vx
                // 8xy5
                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 6) => {
                // Set Vx = Vx SHR1
                // if the least-signigicant bit of Vx is 1, then VF is set to 1, otherwise 0.  THen Vx is divided by 2
                // 8xy6
                let x = d2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            (8, _, _, 7) => {
                // Set Vx = Vy - Vx, set Vx = NOT borrow
                // if Vy > Vx, then VF is set to 1 otherwise 0.  Results stored in Vx
                // 8xy7

                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 0xE) => {
                // Set Vx = Vx SHL 1.
                // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
                // 8xyE
                let x = d2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }
            (9, _, _, 0) => {
                // Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2
                // 9xy0
                let x = d2 as usize;
                let y = d3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // Set I = nnn.
                // The value of register I is set to nnn.
                // Annn
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            (0xB, _, _, _) => {
                // Jump to location nnn + V0.
                // The program counter is set to nnn plus the value of V0.
                // Bnnn
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                // Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
                // The results are stored in Vx. See instruction 8xy2 for more information on AND.
                // Cxkk
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            (0xD, _, _, _) => {
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                // The interpreter reads n bytes from memory, starting at the address stored in I.
                // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
                // Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
                // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
                // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
                // Dxyn

                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[d2 as usize] as u16;
                let y_coord = self.v_reg[d3 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = d4;
                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }
            (0xE, _, 9, 0xE) => {
                // Ex9E
                // Skip if keys pressed
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                //Skip if keys not pressed
                // ExA1
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                // Fx07
                // set Vx to delay timer value
                let x = d2 as usize;
                self.v_reg[x] = self.dt;
            }
            (0xF, _, 0, 0xA) => {
                // Fx0A
                // Wait for key press - blocks until a key is prssed
                // When more than one key prssed, lowest indexed is used.  This key is stored in Vx
                let x = d2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => {
                // Fx15
                // Dt = Vx
                let x = d2 as usize;
                self.dt = self.v_reg[x];
            }
            (0xF, _, 1, 8) => {
                // Fx18
                // St = Vx
                let x = d2 as usize;
                self.st = self.v_reg[x];
            }
            (0xF, _, 1, 0xE) => {
                // Fx1E
                // I += Vx
                // if overflow, register should simply roll over to 0.  (rusts wrapping_add)
                let x = d2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                // Fx29
                // Set I to Font Address
                // fonts are stored in the first sections of ram
                // we are multiplying by 5 since each font is 5 bytes long
                let x = d2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }
            (0xF, _, 3, 3) => {
                // Fx33
                // i = BCD of Vx (BCD - binary coded decimal)
                let x = d2 as usize;
                let vx = self.v_reg[x] as f32;
                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => {
                //Store V0 - VX into I
                // V Registers V0 thru the specified VX (inclusive)
                // with the same range of values from RAM, beginning with the address in the I Register. This first one stores the
                // values into RAM, while the next one will load them the opposite way.
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }
            (0xF, _x, 6, 5) => {
                // Load I into V0 - Vx
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Chip8 {
        Chip8::new()
    }

    #[test]
    fn push_test() {
        let mut c8 = setup();

        c8.push(15);

        assert_eq!(c8.sp, 1);
        assert_eq!(c8.stack[0], 15);
    }

    #[test]
    fn pop_test() {
        let mut c8 = setup();

        c8.push(15);
        assert_eq!(c8.pop(), 15);
        assert_eq!(c8.sp, 0);
    }

    #[test]
    fn reset() {
        let mut c8 = Chip8::new();
        // set random data
        c8.pc += 0x0F;
        c8.ram = [0xF; MEM_SIZE];
        c8.screen = [true; SCREEN_HEIGHT * SCREEN_WIDTH];
        c8.v_reg = [0xF; V_REG_SIZE];
        c8.i_reg = 0xFF;
        c8.sp = 0x1D;
        c8.stack = [0xF; STACK_SIZE];
        c8.keys = [true; KEYPAD_SIZE];
        c8.dt = 0x1D;
        c8.st = 0x1D;

        c8.reset();

        // should be the same as a new chip
        let c8_new = Chip8::new();
        assert_eq!(c8.pc, c8_new.pc);
        assert_eq!(c8.ram, c8_new.ram);
    }
}
