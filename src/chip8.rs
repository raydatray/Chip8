use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTERS_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const KEYS_SIZE: usize = 16;
const FONTSET_SIZE: usize = 80;

const START_ADDRESS: u16 = 0x200;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; REGISTERS_SIZE],
    i_register: u16,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; KEYS_SIZE],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; REGISTERS_SIZE],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; KEYS_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        };
        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emulator
    }

    pub fn get_screen(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx:usize, pressed:bool) {
        self.keys[idx] = pressed;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let begin = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.ram[begin..end].copy_from_slice(data);
    }
    pub fn reset(&mut self){
        self.program_counter = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; REGISTERS_SIZE];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; KEYS_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    //Push the address of a subroutine onto the stack
    fn push(&mut self, address: u16){
        self.stack[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }
    //Pop the address of a subroutine off the stack and return its address
    //Last statement within a fn is assumed to be a return even without keyword
    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    //Timers
    //Modified once every frame
    //Only implementing delay timer, not sound timer
    pub fn timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            //Make a sound (To be implemented)
            self.sound_timer -= 1;
        }
    }

    //CPU Execution per cycle (tick)
    //1. Fetch instruction from RAM at memory address loaded into program counter
    //2. Decode this instruction
    //3. Execute
    //4. Move program counter to next instruction
    pub fn tick(&mut self) {
        let instruction = self.fetch();
        self.execute(instruction);
    }

    //Instructions are held in 16 bytes (HEX)
    //RAM is 8 bytes, therefore each instruction is held side by side
    fn fetch(&mut self) -> u16 {
        let left_byte = self.ram[self.program_counter as usize] as u16;
        let right_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let instruction = (left_byte << 8) | right_byte;
        self.program_counter += 2;
        instruction
    }

    //Execute the instruction from fetch
    //Use MATCH statement
    fn execute(&mut self, instruction: u16) {
        //An instruction looks like XXXX in hex
        //Extract each hex "digit" using bitwise operators
        let digit1 = (instruction & 0xF000) >> 12;
        let digit2 = (instruction & 0x0F00) >> 8;
        let digit3 = (instruction & 0x00F0) >> 4;
        let digit4 = instruction & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            //0000:NOP (Do nothing)
            (0,0,0,0) => return,
            //00E0:Clear screen
            (0,0,0xE,0) => { self.screen = [false; SCREEN_WIDTH*SCREEN_HEIGHT]; },
            //OOEE: Return from subroutine
            (0,0,0xE,0xE) => {
                let return_address = self.pop();
                self.program_counter = return_address;
            },
            //1NNN: Move to address program counter to NNN
            (1,_,_,_) => {
                let nnn = instruction & 0xFFF;
                self.program_counter = nnn;
            },
            //2NNN: Call subroutine. Place current PC into stack, then move PC to NNN
            (2,_,_,_) => {
                let nnn = instruction & 0xFFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            },
            //3XNN: Skip if Vx = NN
            (3,_,_,_) => {
                let x = digit2 as usize;
                let nn = (instruction & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.program_counter += 2;
                }
            },
            //4XNN: Skip if Vx != NN
            (4,_,_,_) => {
                let x = digit2 as usize;
                let nn = (instruction & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.program_counter += 2;
                }
            },
            //5XY0 : Skip if Vx = Vy
            (5,_,_,_) => {
                if self.v_registers[digit2 as usize] == self.v_registers[digit3 as usize] {
                    self.program_counter += 2;
                }
            },
            //6XNN: Vx = NN
            (6,_,_,_) => {
                let nn = (instruction & 0xFF) as u8;
                self.v_registers[digit2 as usize] = nn;
            },
            //7XNN: Vx += NN
            (7,_,_,_) => {
                let nn = (instruction & 0xFF) as u8;
                self.v_registers[digit2 as usize] = self.v_registers[digit2 as usize].wrapping_add(nn);
            },
            //8XY0: Set Vx to Vy
            (8,_,_,0) => {
                self.v_registers[digit2 as usize] = self.v_registers[digit3 as usize];
            },
            //8XY1: Set Vx to Vx OR Vy (bitwise)
            (8,_,_,1) => {
                self.v_registers[digit2 as usize] |= self.v_registers[digit3 as usize];
            },
            //8XY2: Set Vx to Vx AND Vy (bitwise)
            (8,_,_,2) => {
                self.v_registers[digit2 as usize] &= self.v_registers[digit3 as usize];
            },
            //8XY3: Set Vx to Vx XOR Vy (bitwise)
            (8,_,_,3) => {
                self.v_registers[digit2 as usize] ^= self.v_registers[digit3 as usize];
            },
            //8XY4: Vx += Vy. If there is overflow, put carry in Vf(0xF)
            (8,_,_,4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);

                self.v_registers[0xF] = if carry {1} else {0};
                self.v_registers[x] = new_vx;
            },
            //8XY5: Vx -= Vy. If Vx>Vy, put 1 in Vf(0xF)
            (8,_,_,5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);

                self.v_registers[0xF] = if borrow {0} else {1};
                self.v_registers[x] = new_vx;
            },
            //8XY6: If LSB of Vx is 1, put in Vf(0xF). Right shift Vx by 1 bit.
            (8,_,_,6) => {
                self.v_registers[0xF] = self.v_registers[digit2 as usize] & 1;
                self.v_registers[digit2 as usize] >>= 1;
            },
            //8XY7: Vx = Vy-Vx. If Vy>Vx, put 1 in Vf(0xF)
            (8,_,_,7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);

                self.v_registers[0xF] = if borrow {0} else {1};
                self.v_registers[x] = new_vx;
            },
            //8XYE: If MSB of Vx is 1, put in Vf(0xF). Left shift Vx by 1 bit.
            (8,_,_,0xE) => {
                self.v_registers[0xF] = (self.v_registers[digit2 as usize] >> 7) & 1;
                self.v_registers[digit2 as usize] <<= 1;
            },
            //9XY0: Skip of Vx != Vy
            (9,_,_,0) => {
                if self.v_registers[digit2 as usize] != self.v_registers[digit3 as usize]{
                    self.program_counter += 2;
                }
            },
            //ANNN: Set value of Iregister to nnn
            (0xA,_,_,_) => {
                self.i_register = (instruction & 0xFFF);
            },
            //BNNN: Set Program Counter to V[0] + nnn
            (0xB,_,_,_) => {
                self.program_counter = (self.v_registers[0] as u16) + (instruction & 0xFFF);
            },
            //CXKK: Set Vx to a random byte AND kk
            (0xC,_,_,_) => {
                let random: u8 = random();
                self.v_registers[digit2 as usize] = ((instruction & 0xFF) as u8) & random;
            }
            //DXYN: Draw Sprite
            //Sprite: 1 byte wide (8 bits long) starting at (x,y) (held in Vx, Vy)
            //N: Number of pixels tall (starting from address Iregister)
            //Drawing: XORed onto the screen. If there was any collision,Vf =1
            //If sprite "spills" over screen, its wrapped around to the other side of the row
            (0xD,_,_,_) => {
                let x_coord = self.v_registers[digit2 as usize] as u16;
                let y_coord = self.v_registers[digit3 as usize] as u16;
                let height = digit4;
                let mut collision = false;

                for yLine in 0..height {
                    let row_address = self.i_register + yLine as u16;
                    let row_pixels = self.ram[row_address as usize];

                    for xLine in 0..8 {
                        if (row_pixels & (0b1000_0000 >> xLine)) != 0 {
                            //Wrapping
                            let x = (x_coord + xLine) as usize % SCREEN_WIDTH;
                            let y = (y_coord + yLine) as usize % SCREEN_HEIGHT;

                            let screen_index = x + SCREEN_WIDTH * y;
                            collision |= self.screen[screen_index];
                            self.screen[screen_index] ^= true;
                        }
                    }
                }
                if collision {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }
            },
            //EX9E: Skip next instruction if key with the value of Vx is pressed
            (0xE,_,9,0xE) => {
                if self.keys[(self.v_registers[digit2 as usize]) as usize] {
                    self.program_counter += 2;
                }
            },
            //ExA1: Skip next instruction if key with the value of Vx is NOT pressed
            (0xE,_,0xA,1) => {
                if !(self.keys[(self.v_registers[digit2 as usize]) as usize]) {
                    self.program_counter += 2;
                }
            },
            //FX07: Set Vx as delay timer
            (0xF,_,0,7) => {
                self.v_registers[digit2 as usize] = self.delay_timer;
            }
            //FX0A: Wait for a keypress and store it into Vx
            (0xF,_,0,0xA) => {
                let mut pressed = false;
                while !pressed {
                    for i in 0..self.keys.len() {
                        if self.keys[i] {
                            self.v_registers[digit2 as usize] = i as u8;
                            pressed = true;
                            break;
                        }
                    }
                }
            },
            //FX15: Set delay timer as Vx
            (0xF,_,1,5) => {
                self.delay_timer = self.v_registers[digit2 as usize];
            },
            //FX18: Set sound timer as Vx
            (0xF,_,1,8) => {
                self.sound_timer = self.v_registers[digit2 as usize];
            },
            //FX1E: Iregister += Vx
            (0xF,_,1,0xE) => {
                self.i_register = self.i_register.wrapping_add((self.v_registers[digit2 as usize] as u16));
            },
            //FX29: Load sprite into Iregister. E
            //Each sprite is 5 bits long. (Starting at 0)
            (0xF,_,2,9) => {
                let sprite_index = (self.v_registers[digit2 as usize] as u16) * 5;
                self.i_register = sprite_index;
            },
            //FX33: Store BCD of Vx into memory starting from address Iregister
            //Vx: 16 bits -> 2^8 (256)
            //100 -> I, 10 -> I+1, 1 -> I+2
            (0xF,_,3,3) => {
                self.ram[self.i_register as usize] = self.v_registers[digit2 as usize] / 100;
                self.ram[(self.i_register as usize) + 1] = (self.v_registers[digit2 as usize] / 10) % 10;
                self.ram[(self.i_register as usize) + 2] = self.v_registers[digit2 as usize] % 10;
            },
            //FX55: Copy values of V0 to Vx into memory starting at address in Iregister
            (0xF,_,5,5) => {
                let start_address = self.i_register as usize;
                for i in 0..=digit2 as usize{
                    self.ram[start_address + i] = self.v_registers[i];
                }
            },
            //FX65: Read values into V0 to Vx from memory starting at address in Iregister
            (0xF,_,6,5) => {
                let start_address = self.i_register as usize;
                for i in 0..=digit2 as usize{
                    self.v_registers[i] = self.ram[start_address + i];
                }
            },
            (_,_,_,_) => unimplemented!("Unimplemented Instruction: {}", instruction),
        }
    }
}