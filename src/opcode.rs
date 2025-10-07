#[derive(Clone, Debug)]
pub(crate) struct OpCode<const BYTES: usize> {
    code: BitData,
    bits: [BitData; BYTES],
}

#[derive(Clone, Copy, Debug, Default)]
struct BitData {
    bits: u8,
}

pub(crate) const MOV: OpCode<5> = OpCode::new(
    BitData::new(6),
    [
        BitData::new(1), // d
        BitData::new(1), // w
        BitData::new(2), // mod
        BitData::new(3), // reg
        BitData::new(3), // rm
    ],
);

impl<const BYTES: usize> OpCode<BYTES> {
    const fn new(code: BitData, bits: [BitData; BYTES]) -> Self {
        Self { code, bits }
    }

    pub(crate) fn extract(
        &self,
        byte_reader: &mut impl crate::reader::ByteReaderT,
    ) -> Result<[u8; BYTES], crate::reader::Error> {
        let mut bits = [0; BYTES];
        let mut byte = byte_reader.byte_read_next()?.ok_or(crate::reader::Error::Empty)?;
        let mut offset = 8 - self.code.bits;

        for i in 0..BYTES {
            if offset == 0 {
                offset = 8;
                byte = byte_reader.byte_read_next()?.ok_or(crate::reader::Error::Empty)?;
            }

            offset -= self.bits[i].bits;

            let mask = ((1 << self.bits[i].bits) - 1) << offset;
            let data = (byte & mask) >> offset;

            bits[i] = data;
        }

        Ok(bits)
    }
}

impl BitData {
    const fn new(bits: u8) -> Self {
        Self { bits }
    }
}

#[cfg(test)]
mod fixtures {
    #[rstest::fixture]
    pub(crate) fn byte_reader(
        #[default([0])] bits: impl IntoIterator<Item = u8>,
    ) -> crate::reader::ByteReaderForTesting {
        let data = bits.into_iter().collect();
        crate::reader::ByteReaderForTesting::new(data)
    }
}

#[cfg(test)]
mod test {
    use super::fixtures::*;
    use super::*;

    #[rstest::rstest]
    fn opcode_extract(#[with([0b10000010, 0b10100100])] mut byte_reader: crate::reader::ByteReaderForTesting) {
        let bits = MOV.extract(&mut byte_reader).unwrap();
        assert_eq!(bits, [0b1, 0b0, 0b10, 0b100, 0b100]);
    }
}
