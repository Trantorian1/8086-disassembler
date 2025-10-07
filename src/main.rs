use std::io::{Read as _, Write as _};

use crate::reader::ByteReaderT;

mod opcode;
mod reader;

enum Error {
    InvalidArguments,
    InvalidFile(std::io::Error),
    InvalidRead(reader::Error),
    InvalidWrite(std::io::Error),
    OpcodeUnsupported(u8),
    OpcodeMalformed(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArguments => write!(f, "Invalid arguments, call with the name of the file to disassemble"),
            Self::InvalidFile(err) => write!(f, "Failed to open file: {err}"),
            Self::InvalidRead(err) => write!(f, "Failed to read file: {err}"),
            Self::InvalidWrite(err) => write!(f, "Failed to write to stdout: {err}"),
            Self::OpcodeUnsupported(opcode) => write!(f, "Unsupported opcode in first byte: {opcode:b}"),
            Self::OpcodeMalformed(err) => write!(f, "Malformed opcode: {err}"),
        }
    }
}

const OPCODE_ARITHMETIC: [&str; 8] = [
    "add", // 0b000
    "adc", // 0b001
    "NIL", // 0b010
    "sbb", // 0b011
    "NIL", // 0b100
    "sub", // 0b101
    "NIL", // 0b110
    "cmp", // 0b111
];

const REGISTERS_8_BIT: [&str; 8] = [
    "al", // 0b000
    "cl", // 0b001
    "dl", // 0b010
    "bl", // 0b011
    "ah", // 0b100
    "ch", // 0b101
    "dh", // 0b110
    "bh", // 0b111
];

const REGISTERS_16_BIT: [&str; 8] = [
    "ax", // 0b000
    "cx", // 0b001
    "dx", // 0b010
    "bx", // 0b011
    "sp", // 0b100
    "bp", // 0b101
    "si", // 0b110
    "di", // 0b111
];

const MEMORY_ADDR: [&str; 8] = [
    "bx + si", // 0b000
    "bx + di", // 0b001
    "bp + si", // 0b010
    "bp + di", // 0b011
    "si",      // 0b100
    "di",      // 0b101
    "bp",      // 0b110
    "bx",      // 0b111
];

fn main() -> Result<(), Error> {
    let path = std::env::args().skip(1).next().ok_or(Error::InvalidArguments)?;
    let file = std::fs::File::open(path).map_err(Error::InvalidFile)?;
    let mut reader = reader::ByteReader::new(file).map_err(Error::InvalidRead)?;

    let mut out = std::io::stdout();

    while let Some(byte) = reader.byte_read().map_err(Error::InvalidRead)? {
        match byte & 0b11111100 {
            //
            // ┌───────────────────────────────┐
            // │REGISTER/MEMORY MOV INSTRUCTION│
            // └───────────────────────────────┘
            //
            //  byte_1   byte_2   disp_lo  disp_hi
            //
            // 10001011 11001001 xxxxxxxx xxxxxxxx
            // └┬───┘││ └┤└┬┘└┬┘ └┬─────┘ └┬─────┘
            //  │    ││  │ │  │   │        └───────► (DISP-HI): high displacement bits
            //  │    ││  │ │  │   └────────────────► (DISP-LO): low displacement bits
            //  │    ││  │ │  └────────────────────► (RM.....): register/memory address
            //  │    ││  │ └───────────────────────► (REG....): register address
            //  │    ││  └─────────────────────────► (MOD....): modifier
            //  │    │└────────────────────────────► (W......): width
            //  │    └─────────────────────────────► (D......): direction
            //  └──────────────────────────────────► (OP.....): opcode
            //
            0b10001000 => {
                let [d, w, mode, reg, rm] = opcode::MOV.extract(&mut reader).map_err(Error::InvalidRead)?;

                // Register to register move
                if mode == 0b11 {
                    let (reg_str, rm_str) = if w == 0b0 {
                        (REGISTERS_8_BIT[reg as usize], REGISTERS_8_BIT[rm as usize])
                    } else {
                        (REGISTERS_16_BIT[reg as usize], REGISTERS_16_BIT[rm as usize])
                    };

                    let err = if d == 0b0 {
                        writeln!(out, "mov {rm_str}, {reg_str}")
                    } else {
                        writeln!(out, "mov {reg_str}, {rm_str}")
                    };

                    err.map_err(Error::InvalidWrite)?;
                } else {
                    todo!()
                }
            }
            byte => todo!(),
        };
    }

    // // TODO: move this to something more efficient in the future. We probably don't want to be
    // // reading one byte a time. Reading a page at a time seems like a better idea but then we would
    // // have to handle edge cases where we need a byte after the section which we have just read.
    // while let Some(data) = bytes.next() {
    //     let byte_1 = data.map_err(Error::InvalidRead)?;
    //
    //     match byte_1 & 0b11111100 {
    //         opcode::MOV => {
    //             //
    //             // ┌───────────────────────────────┐
    //             // │REGISTER/MEMORY MOV INSTRUCTION│
    //             // └───────────────────────────────┘
    //             //
    //             //  byte_1   byte_2   disp_lo  disp_hi
    //             //
    //             // 10001011 11001001 xxxxxxxx xxxxxxxx
    //             // └┬───┘││ └┤└┬┘└┬┘ └┬─────┘ └┬─────┘
    //             //  │    ││  │ │  │   │        └───────► (DISP-HI): high displacement bits
    //             //  │    ││  │ │  │   └────────────────► (DISP-LO): low displacement bits
    //             //  │    ││  │ │  └────────────────────► (RM.....): register/memory address
    //             //  │    ││  │ └───────────────────────► (REG....): register address
    //             //  │    ││  └─────────────────────────► (MOD....): modifier
    //             //  │    │└────────────────────────────► (D......): direction
    //             //  │    └─────────────────────────────► (W......): width
    //             //  └──────────────────────────────────► (OP.....): opcode
    //             //
    //
    //             opcode_pattern_reg_mem("mov", byte_1, &mut bytes, &mut out)?;
    //         }
    //         opcode::ADD => {
    //             //
    //             // ┌───────────────────────────────┐
    //             // │REGISTER/MEMORY ADD INSTRUCTION│
    //             // └───────────────────────────────┘
    //             //
    //             //  byte_1   byte_2   disp_lo  disp_hi
    //             //
    //             // 10001011 11001001 xxxxxxxx xxxxxxxx
    //             // └┬───┘││ └┤└┬┘└┬┘ └┬─────┘ └┬─────┘
    //             //  │    ││  │ │  │   │        └───────► (DISP-HI): high displacement bits
    //             //  │    ││  │ │  │   └────────────────► (DISP-LO): low displacement bits
    //             //  │    ││  │ │  └────────────────────► (RM.....): register/memory address
    //             //  │    ││  │ └───────────────────────► (REG....): register address
    //             //  │    ││  └─────────────────────────► (MOD....): modifier
    //             //  │    │└────────────────────────────► (D......): direction
    //             //  │    └─────────────────────────────► (W......): width
    //             //  └──────────────────────────────────► (OP.....): opcode
    //             //
    //
    //             opcode_pattern_reg_mem("add", byte_1, &mut bytes, &mut out)?;
    //         }
    //         opcode::SUB => todo!(),
    //         opcode::CMP => todo!(),
    //         opcode::ADD_SUB_CMP_IMM => {
    //             let byte_2 = bytes
    //                 .next()
    //                 .ok_or_else(|| Error::OpcodeMalformed("Missing second byte in add/sub/cmp directive".to_string()))?
    //                 .map_err(Error::InvalidRead)?;
    //             let disp_lo = bytes
    //                 .next()
    //                 .ok_or_else(|| Error::OpcodeMalformed("Missing second byte in add/sub/cmp directive".to_string()))?
    //                 .map_err(Error::InvalidRead)?;
    //             let disp_hi = bytes
    //                 .next()
    //                 .ok_or_else(|| Error::OpcodeMalformed("Missing second byte in add/sub/cmp directive".to_string()))?
    //                 .map_err(Error::InvalidRead)?;
    //             let data_1 = bytes
    //                 .next()
    //                 .ok_or_else(|| Error::OpcodeMalformed("Missing second byte in add/sub/cmp directive".to_string()))?
    //                 .map_err(Error::InvalidRead)?;
    //
    //             let sign = byte_1 & 0b00000010;
    //             let width = byte_1 & 0b00000001;
    //
    //             let mode = byte_2 & 0b11000000;
    //             let opcode = byte_2 & 0b00111000;
    //             let rm = byte_2 & 0b00000111;
    //
    //             // Extract opcode from identifier
    //             let opcode = OPCODE_ARITHMETIC[opcode as usize];
    //
    //             // Retrieve immediate high bits
    //             let data_2 = (width == 0b1)
    //                 .then(|| {
    //                     bytes
    //                         .next()
    //                         .ok_or_else(|| {
    //                             Error::OpcodeMalformed("Missing second byte in add/sub/cmp directive".to_string())
    //                         })?
    //                         .map_err(Error::InvalidRead)
    //                 })
    //                 .unwrap_or(Ok(0))?;
    //
    //             // Immediate to register
    //             if mode == mode::REGISTER {
    //                 let rm_str = if width == 0 {
    //                     REGISTERS_8_BIT[rm as usize]
    //                 } else {
    //                     REGISTERS_16_BIT[rm as usize]
    //                 };
    //
    //                 // Unsigned add
    //                 let err = if sign == 0 {
    //                     let data = u16::from_ne_bytes([data_1, data_2]);
    //                     writeln!(out, "{opcode} {rm_str}, {data}")
    //                 }
    //                 // Signed add
    //                 else {
    //                     let data_signed = i16::from_ne_bytes([data_1, data_2]);
    //                     writeln!(out, "{opcode} {rm_str}, {data_signed}")
    //                 };
    //
    //                 err.map_err(Error::InvalidWrite)?;
    //             }
    //             // Immediate to memory
    //             else {
    //                 let rm_str = MEMORY_ADDR[rm as usize];
    //                 let width_str = if width == 0 { "byte" } else { "word" };
    //
    //                 // Memory add
    //                 let err = if mode == mode::MEMORY {
    //                     // Unsigned add
    //                     if sign == 0 {
    //                         let data = ((data_2 as u16) << 8) | (data_1 as u16);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str}], {data}")
    //                     }
    //                     // Signed add
    //                     else {
    //                         let data_signed = i16::from_ne_bytes([data_2, data_1]);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str}], {data_signed}")
    //                     }
    //                 }
    //                 // Add with 8-bit displacement
    //                 else if mode == mode::DISPLACEMENT_8BIT {
    //                     // Unsigned add
    //                     if sign == 0 {
    //                         let data = ((data_2 as u16) << 8) | (data_1 as u16);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str} + {disp_lo}], {data}")
    //                     }
    //                     // Signed add
    //                     else {
    //                         let data_signed = i16::from_ne_bytes([data_2, data_1]);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str} + {disp_lo}], {data_signed}")
    //                     }
    //                 }
    //                 // Add with 16-bit displacement
    //                 else {
    //                     let disp = ((disp_hi as u16) << 8) | (disp_lo as u16);
    //
    //                     // Unsigned add
    //                     if sign == 0 {
    //                         let data = u16::from_ne_bytes([data_2, data_1]);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str} + {disp}], {data}")
    //                     }
    //                     // Signed add
    //                     else {
    //                         let data_signed = i16::from_ne_bytes([data_2, data_1]);
    //                         writeln!(out, "{opcode} {width_str} [{rm_str} + {disp}], {data_signed}")
    //                     }
    //                 };
    //
    //                 err.map_err(Error::InvalidWrite)?;
    //             }
    //         }
    //         _ => match byte_1 & 0b11110000 {
    //             opcode::MOV_IMM => {
    //                 //
    //                 // ┌─────────────────────────────────────┐
    //                 // │IMMEDIATE TO REGISTER MOV INSTRUCTION│
    //                 // └─────────────────────────────────────┘
    //                 //
    //                 //  byte_1   data_1   data_2
    //                 //
    //                 // 10111011 xxxxxxxx xxxxxxxx
    //                 // └┬──┘│├┘ └┬─────┘ └┬─────┘
    //                 //  │   ││   │        └───────► (DATA): high immediate bits, if w=1
    //                 //  │   ││   └────────────────► (DATA): low immediate bits
    //                 //  │   │└────────────────────► (REG.): regsiter address
    //                 //  │   └─────────────────────► (W...): width
    //                 //  └─────────────────────────► (OP..): opcode
    //                 //
    //
    //                 let data_1 = bytes
    //                     .next()
    //                     .ok_or(Error::OpcodeMalformed(
    //                         "Missing first data byte in 8-bit immediate MOV directive".to_string(),
    //                     ))?
    //                     .map_err(Error::InvalidRead)?;
    //
    //                 let width = byte_1 & 0b00001000;
    //                 let reg = byte_1 & 0b00000111;
    //
    //                 let err = if width == (Width::Byte as u8) << 3 {
    //                     let reg_str = REGISTERS_8_BIT[reg as usize];
    //                     writeln!(out, "mov {reg_str}, {data_1}")
    //                 } else {
    //                     let reg_str = REGISTERS_16_BIT[reg as usize];
    //
    //                     let data_2 = bytes
    //                         .next()
    //                         .ok_or(Error::OpcodeMalformed(
    //                             "Missing second data byte in 16-bit immediate MOV directive".to_string(),
    //                         ))?
    //                         .map_err(Error::InvalidRead)?;
    //                     let data = ((data_2 as u16) << 8) | (data_1 as u16);
    //
    //                     writeln!(out, "mov {reg_str}, {data}")
    //                 };
    //
    //                 err.map_err(Error::InvalidWrite)?;
    //             }
    //             byte => return Err(Error::OpcodeUnsupported(byte)),
    //         },
    //     }
    // }
    //
    // out.flush().map_err(Error::InvalidWrite)?;

    Ok(())
}

// /// Register/memory to register
// fn opcode_pattern_reg_mem(
//     opcode: &'static str,
//     byte_1: u8,
//     bytes: &mut std::io::Bytes<impl std::io::Read>,
//     out: &mut impl std::io::Write,
// ) -> Result<(), Error> {
//     let byte_2 = bytes
//         .next()
//         .ok_or_else(|| {
//             Error::OpcodeMalformed(format!(
//                 "Missing second byte in {} directive",
//                 opcode.to_ascii_uppercase()
//             ))
//         })?
//         .map_err(Error::InvalidRead)?;
//
//     let dest = byte_1 & 0b00000010;
//     let width = byte_1 & 0b00000001;
//     let mode = byte_2 & 0b11000000;
//     let reg = (byte_2 & 0b00111000) >> 3;
//     let rm = byte_2 & 0b00000111;
//
//     let reg_str = if width == Width::Byte as u8 {
//         REGISTERS_8_BIT[reg as usize]
//     } else {
//         REGISTERS_16_BIT[reg as usize]
//     };
//
//     // Register to register move
//     if mode == Mode::Register as u8 {
//         let rm_str = if width == Width::Byte as u8 {
//             REGISTERS_8_BIT[rm as usize]
//         } else {
//             REGISTERS_16_BIT[rm as usize]
//         };
//
//         let err = if dest == Dest::Rm as u8 {
//             writeln!(out, "{opcode} {rm_str}, {reg_str}")
//         } else {
//             writeln!(out, "{opcode} {reg_str}, {rm_str}")
//         };
//
//         err.map_err(Error::InvalidWrite)
//     }
//     // Memory to register / register to memory move
//     else {
//         let rm_str = MEMORY_ADDR[rm as usize];
//
//         // Memory move
//         let err = if mode == Mode::Memory as u8 {
//             if dest == Dest::Rm as u8 {
//                 writeln!(out, "mov [{rm_str}], {reg_str}")
//             } else {
//                 writeln!(out, "mov {reg_str}, [{rm_str}]")
//             }
//         }
//         // Move with 8-bit displacement
//         else if mode == Mode::Displacement8bit as u8 {
//             let disp_lo = bytes
//                 .next()
//                 .ok_or(Error::OpcodeMalformed(format!(
//                     "Missing disp_lo byte in {} directive targettng memory",
//                     opcode.to_ascii_uppercase()
//                 )))?
//                 .map_err(Error::InvalidRead)?;
//
//             if dest == Dest::Rm as u8 {
//                 if disp_lo == 0 {
//                     writeln!(out, "{opcode} [{rm_str}], {reg_str}")
//                 } else {
//                     writeln!(out, "{opcode} [{rm_str} + {disp_lo}], {reg_str}")
//                 }
//             } else {
//                 if disp_lo == 0 {
//                     writeln!(out, "{opcode} {reg_str}, [{rm_str}]")
//                 } else {
//                     writeln!(out, "{opcode} {reg_str}, [{rm_str} + {disp_lo}]")
//                 }
//             }
//         }
//         // Move with 16-bit displacement
//         else {
//             let disp_lo = bytes
//                 .next()
//                 .ok_or(Error::OpcodeMalformed(format!(
//                     "Missing disp_lo byte in {} directive targettng memory",
//                     opcode.to_ascii_uppercase()
//                 )))?
//                 .map_err(Error::InvalidRead)?;
//             let disp_hi = bytes
//                 .next()
//                 .ok_or(Error::OpcodeMalformed(format!(
//                     "Missing disp_hi byte in {} directive targettng memory",
//                     opcode.to_ascii_uppercase()
//                 )))?
//                 .map_err(Error::InvalidRead)?;
//             let disp = ((disp_hi as u16) << 8) | (disp_lo as u16);
//
//             if dest == Dest::Rm as u8 {
//                 writeln!(out, "{opcode} [{rm_str} + {disp}], {reg_str}")
//             } else {
//                 writeln!(out, "{opcode} {reg_str}, [{rm_str} + {disp}]")
//             }
//         };
//
//         err.map_err(Error::InvalidWrite)
//     }
// }
