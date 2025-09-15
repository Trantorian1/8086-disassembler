use std::io::{Read as _, Write as _};

enum Error {
    InvalidArguments,
    InvalidFile(std::io::Error),
    InvalidRead(std::io::Error),
    InvalidWrite(std::io::Error),
    OpcodeUnsupported(u8),
    OpcodeMalformed(&'static str),
    OpcodeUnsupportedMode(u8),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArguments => write!(f, "Invalid arguments, call with the name of the file to disassemble"),
            Self::InvalidFile(err) => write!(f, "Failed to open file: {err}"),
            Self::InvalidRead(err) => write!(f, "Failed to read file: {err}"),
            Self::InvalidWrite(err) => write!(f, "Failed to write to stdout: {err}"),
            Self::OpcodeUnsupported(opcode) => write!(f, "Unsupported opcode: {opcode:b}"),
            Self::OpcodeMalformed(err) => write!(f, "Malformed opcode: {err}"),
            Self::OpcodeUnsupportedMode(mode) => write!(f, "Unsupported opcode mode: {mode:b}"),
        }
    }
}

#[repr(u8)]
enum OpCode {
    MOVE = 0b10001000,
}

#[repr(u8)]
#[allow(unused)]
enum Dest {
    Rm = 0b0,
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢿⠫⡁⢀⣀⣠⣒⣬⠿⡳⠛⠏⣿⢿⣛⢷⣄⡀⠀⠀⢀⡽⣻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣻⣽⣿⣾⡿⣽⣾⣿⣽⣿⠟⣥⢞⡩⣴⡿⢗⣫⣭⣴⣷⣿⣶⣿⣿⣿⣿⣿⣏⠼⢆⣀⢦⣝⡿⣽⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⡿⣷⣿⣿⣯⣿⣿⠋⣨⡴⣫⣾⣿⣿⢿⣫⣷⣿⣿⣿⣿⣿⡿⡿⢟⡟⣻⣶⣾⣾⣓⣾⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣻
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠏⣨⢋⣾⣿⡷⣻⣵⣿⣿⣿⣿⣿⣿⣿⣯⢷⣻⢯⣜⠦⠹⣿⣿⣿⡜⣿⣿⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⢣⣿⡟⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣽⢻⣯⣯⡜⠀⠁⠘⣿⣿⣯⣽⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⣰⡼⢫⣾⣿⡟⣽⣿⣿⣿⣿⣿⣿⣽⣞⣿⣻⣾⣱⢂⠀⠀⠀⢿⣿⣿⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢀⡋⠰⣱⣿⢏⣾⣿⣿⣿⣿⣿⣯⣿⢿⡾⣿⣵⢯⡗⠂⠀⠀⠀⠸⢿⣿⣾⣿⣿⣿⣿⡇⠀⠘⠀⠀⡏⠉⣉⣹⠉⢹⣿⠉⢹⣿⠋⢩⠉⢻⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢠⠇⡱⣿⠏⣾⡿⢻⣿⣿⣿⣿⣿⣾⢫⢝⡺⡝⣺⢼⣷⣶⠞⣆⠠⣛⣿⣿⣿⣿⣿⣿⣇⣀⣰⣀⣠⣇⣀⣒⣿⣀⣈⣹⣀⣈⣹⣀⡸⣀⣼⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣈⣷⡠⢹⣶⠓⢀⣼⣻⣿⣾⣿⣯⣟⣯⣶⣷⠈⣴⣯⢿⣶⢧⡄⠀⡛⡟⠁⠈⠐⠉⡙⢯⡟⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣭⣷⠀⣌⡟⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣆⠻⣿⣾⠟⠋⠁⠀⠝⡄⠀⠀⠀⡀⣠⠒⠛⠓⢛⠛⠛⡛⠛⡟⠛⣛⣻⠛⢛⠛⣿⠛⢛⣛⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡎⣞⣧⠨⠀⠀⢙⣿⣿⣿⣾⣿⣿⣿⣿⣿⣿⡇⡀⠈⣯⢣⡰⠀⠀⢸⣿⣿⣿⣿⣷⣿⣿⠀⢸⣿⠀⠠⡄⠀⡇⠀⠭⣿⠀⢨⠀⣻⠀⠨⢽⣿⣿⣿⣿
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣝⣿⣝⠨⠀⠰⣻⣿⣻⣿⢿⣿⣿⣿⣿⡿⠇⠁⢠⡛⠆⠁⠀⠀⢺⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣿⡛⡟⢿⠿⣿⣿⣿⣿⡬⣾⣗⡀⠀⢱⣿⣳⣯⣟⣯⢷⣿⣿⣿⣯⣵⣿⡂⢟⡀⠀⠀⠀⢸⣿⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⣿⣿⣿⣿⣿⣿
    /// ⣿⣿⣷⣿⣌⣧⢋⡜⠹⢿⣿⣧⡌⢹⡈⠀⠀⣿⣟⡾⣽⣾⣿⣿⣿⣿⣿⣿⣟⢫⠖⡂⠀⠀⢰⣿⣯⣻⣽⣟⡮⢳⠹⣞⡷⣯⢷⡻⣾⡽⣞⡷⣯⣟⣞⣳⢻⣜⡳⣝⡲⣝⢮⡳⣛
    /// ⣿⣿⣿⣿⣿⣿⣇⡄⠃⠄⠛⢿⣧⠸⡇⠀⢤⢻⣿⣿⢿⡿⣿⣻⣿⣿⣿⣿⣿⣿⡀⡠⢘⠧⣼⣿⣿⣿⣿⣟⡠⠃⠤⢘⢻⣟⡿⣧⣟⡿⣟⣿⢧⣼⣼⡸⠿⡼⢧⡻⣿⣻⣧⣿⣣
    /// ⣿⣿⣿⣿⣿⣿⣿⣾⡥⠀⠀⠀⠙⠄⡇⢀⠀⢹⢺⠓⣜⣿⣿⣿⣿⣿⣿⠿⠛⢫⣽⡁⠁⣾⠟⡟⠿⣿⣿⣿⣷⣧⠀⠀⠐⡘⢻⠳⣟⣿⡻⠏⠛⠱⠊⠱⠁⠂⠀⠡⡙⠿⣽⣷⡻
    /// ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡀⠀⠀⠀⠌⠀⣻⢀⠇⣲⣿⣿⣿⣷⣿⢾⣽⣶⠅⠼⣿⣝⢠⡇⠫⠀⠁⠀⠙⢿⣿⣿⡀⢀⠰⣈⠄⠑⢌⠳⡝⠀⠀⠀⠀⠀⢃⠀⠀⢢⡱⢩⠾⣝⠣
    /// ⣿⣿⣿⣿⣿⣿⣿⡿⠛⠝⠛⠈⠈⡦⢸⣄⠚⣧⠁⠻⡿⣿⣿⣿⣿⡿⣯⢟⣎⣻⡿⣇⢶⡧⠀⠡⠀⡀⠀⠠⠙⠢⠙⡄⢻⣜⣮⢤⣀⠫⠀⠀⠀⠀⠀⠀⠀⠀⣈⠳⠌⠡⠚⣍⠂
    /// ⡿⠿⠛⠋⠉⠉⠀⠀⡀⣀⢀⢢⣼⡀⢸⣯⣿⣿⣦⠠⠕⢿⣿⣿⣿⣿⣽⡿⣞⢇⡿⣣⣿⠃⠀⠀⠀⢀⡁⠒⠀⠀⠘⡸⢷⣾⣿⣷⣮⣇⢧⣎⡥⠀⠀⣐⢢⣄⢠⡑⣌⡲⡝⠄⠀
    Reg = 0b1,
}

#[repr(u8)]
#[allow(unused)]
enum Width {
    Byte = 0b0,
    Word = 0b1,
}

#[repr(u8)]
#[allow(unused)]
enum Mode {
    Memory = 0b00000000,
    Displacement8bit = 0b01000000,
    Displacement16bit = 0b10000000,
    Register = 0b11000000,
}

const REGISTERS_8_BIT: [&str; 8] = [
    "AL", // 0b000
    "CL", // 0b001
    "DL", // 0b010
    "BL", // 0b011
    "AH", // 0b100
    "CH", // 0b101
    "DH", // 0b110
    "BH", // 0b111
];

const REGISTERS_16_BIT: [&str; 8] = [
    "AX", // 0b000
    "CX", // 0b001
    "DX", // 0b010
    "BX", // 0b011
    "SP", // 0b100
    "BP", // 0b101
    "SI", // 0b110
    "DI", // 0b111
];

const MEMORY_ADDR: [&str; 8] = [
    "BX + SI", // 0b000
    "BX + DI", // 0b001
    "BP + SI", // 0b010
    "BP + DI", // 0b011
    "SI",      // 0b100
    "DI",      // 0b101
    "BP",      // 0b110
    "BX",      // 0b111
];

fn main() -> Result<(), Error> {
    let path = std::env::args().skip(1).next().ok_or(Error::InvalidArguments)?;
    let file = std::fs::File::open(path).map_err(Error::InvalidFile)?;
    let read = std::io::BufReader::new(file);
    let mut bytes = read.bytes();

    let mut out = std::io::BufWriter::new(std::io::stdout());

    // TODO: move this to something more efficient in the future. We probably don't want to be
    // reading one byte a time. Reading a page at a time seems like a better idea but then we would
    // have to handle edge cases where we need a byte after the section which we have just read.
    while let Some(data) = bytes.next() {
        let byte_1 = data.map_err(Error::InvalidRead)?;
        let opcode = byte_1 & 0b11111100;

        if opcode == OpCode::MOVE as u8 {
            let byte_2 = bytes
                .next()
                .ok_or(Error::OpcodeMalformed("Missing second byte in MOV directive"))?
                .map_err(Error::InvalidRead)?;

            //
            // ┌───────────────────────────────┐
            // │REGISTER/MEMORY MOV INSTRUCTION│
            // └───────────────────────────────┘
            //
            //  byte_1   byte_2   disp_lo  disp_hi
            //
            // 10001011 11001001 00000000 00000000
            // └┬───┘││ └┤└┬┘└┬┘ └┬─────┘ └┬─────┘
            //  │    ││  │ │  │   │        └───────► (DISP-HI): high displacement bits
            //  │    ││  │ │  │   └────────────────► (DISP-LO): low displacement bits
            //  │    ││  │ │  └────────────────────► (RM.....): register/memory address
            //  │    ││  │ └───────────────────────► (REG....): register address
            //  │    ││  └─────────────────────────► (MOD....): modifier
            //  │    │└────────────────────────────► (D......): direction
            //  │    └─────────────────────────────► (W......): width
            //  └──────────────────────────────────► (OP.....): opcode
            //
            let dest = byte_1 & 0b00000010;
            let width = byte_1 & 0b00000001;
            let mode = byte_2 & 0b11000000;
            let reg = (byte_2 & 0b00111000) >> 3;
            let rm = byte_2 & 0b00000111;

            let reg_str = if width == Width::Byte as u8 {
                REGISTERS_8_BIT[reg as usize]
            } else {
                REGISTERS_16_BIT[reg as usize]
            };

            // Register to register move
            if mode == Mode::Register as u8 {
                let rm_str = if width == Width::Byte as u8 {
                    REGISTERS_8_BIT[rm as usize]
                } else {
                    REGISTERS_16_BIT[rm as usize]
                };

                let err = if dest == Dest::Rm as u8 {
                    write!(out, "MOV {rm_str}, {reg_str}\n")
                } else {
                    write!(out, "MOV {reg_str}, {rm_str}\n")
                };

                err.map_err(Error::InvalidWrite)?;
            }
            // Memory to register / register to memory move
            else {
                let rm_str = MEMORY_ADDR[rm as usize];

                // Memory move
                let err = if mode == Mode::Memory as u8 {
                    if dest == Dest::Reg as u8 {
                        write!(out, "MOV {reg_str}, [{rm_str}]")
                    } else {
                        write!(out, "MOV [{rm_str}], {reg_str}")
                    }
                }
                // Move with 8-bit displacement
                else if mode == Mode::Displacement8bit as u8 {
                    let disp_lo = bytes
                        .next()
                        .ok_or(Error::OpcodeMalformed(
                            "Missing disp_lo byte in MOV directive targettng memory",
                        ))?
                        .map_err(Error::InvalidRead)?;

                    if dest == Dest::Reg as u8 {
                        write!(out, "MOV {reg_str}, [{rm_str} + {disp_lo}]")
                    } else {
                        write!(out, "MOV [{rm_str} + {disp_lo}], {reg_str}")
                    }
                }
                // Move with 16-bit displacement
                else {
                    let disp_lo = bytes
                        .next()
                        .ok_or(Error::OpcodeMalformed(
                            "Missing disp_lo byte in MOV directive targettng memory",
                        ))?
                        .map_err(Error::InvalidRead)?;
                    let disp_hi = bytes
                        .next()
                        .ok_or(Error::OpcodeMalformed(
                            "Missing disp_hi byte in MOV directive targettng memory",
                        ))?
                        .map_err(Error::InvalidRead)?;
                    let disp = (disp_hi << 8) & disp_lo;

                    if dest == Dest::Reg as u8 {
                        write!(out, "MOV {reg_str}, [{rm_str} + {disp}]")
                    } else {
                        write!(out, "MOV [{rm_str} + {disp}], {reg_str}")
                    }
                };

                err.map_err(Error::InvalidWrite)?;
            }
        } else {
            return Err(Error::OpcodeUnsupported(opcode));
        }
    }

    out.flush().map_err(Error::InvalidWrite)?;

    Ok(())
}
