#[cfg(test)]
pub use test::*;

#[derive(Debug)]
pub(crate) enum Error {
    Empty,
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Tried to read from empty reader"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) const BUFFER_SIZE: usize = 512;

pub(crate) trait ByteReaderT {
    fn byte_read(&mut self) -> Result<Option<u8>, Error>;
    fn byte_read_next(&mut self) -> Result<Option<u8>, Error>;
}

pub(crate) struct ByteReader<R: std::io::Read> {
    bytes: [u8; BUFFER_SIZE],
    index: usize,
    available: usize,
    reader: R,
}

impl<R: std::io::Read> ByteReader<R> {
    pub(crate) fn new(mut reader: R) -> Result<Self, Error> {
        let mut bytes = [0; BUFFER_SIZE];
        let available = reader.read(&mut bytes).map_err(Error::Io)?;

        if available == 0 {
            Err(Error::Empty)
        } else {
            Ok(Self {
                bytes,
                index: 0,
                available,
                reader,
            })
        }
    }
}

impl<R: std::io::Read> ByteReaderT for ByteReader<R> {
    fn byte_read(&mut self) -> Result<Option<u8>, Error> {
        if self.index < self.available {
            Ok(Some(self.bytes[self.index]))
        } else {
            self.available = self.reader.read(&mut self.bytes).map_err(Error::Io)?;
            if self.available == 0 {
                Ok(None)
            } else {
                self.index = 0;
                Ok(Some(self.bytes[self.index]))
            }
        }
    }

    fn byte_read_next(&mut self) -> Result<Option<u8>, Error> {
        if self.index < self.available {
            let byte = Ok(Some(self.bytes[self.index]));
            self.index += 1;
            byte
        } else {
            self.available = self.reader.read(&mut self.bytes).map_err(Error::Io)?;
            if self.available == 0 {
                self.index = 0;
                Ok(None)
            } else {
                self.index = 0;
                let byte = Ok(Some(self.bytes[self.index]));
                self.index += 1;
                byte
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::reader::ByteReaderT;

    pub struct ByteReaderForTesting {
        data: Vec<u8>,
        index: usize,
    }

    impl ByteReaderForTesting {
        pub fn new(data: Vec<u8>) -> Self {
            Self { data, index: 0 }
        }

        pub fn append_data(&mut self, data: &[u8]) {
            self.data.extend_from_slice(data);
        }
    }

    impl ByteReaderT for ByteReaderForTesting {
        fn byte_read(&mut self) -> Result<Option<u8>, super::Error> {
            Ok(Some(self.data[self.index]))
        }

        fn byte_read_next(&mut self) -> Result<Option<u8>, super::Error> {
            let byte = self.data[self.index];
            self.index += 1;
            Ok(Some(byte))
        }
    }
}
