use file_offset::FileExt as FileOffsetExt;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;

use num::Cast;

struct SeekOverflow(());

pub fn seek_overflow() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, SeekOverflow(()))
}

impl fmt::Debug for SeekOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SeekOverflow").finish()
    }
}

impl error::Error for SeekOverflow {}

impl fmt::Display for SeekOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("overflow while calculating seek offset")
    }
}

pub trait ReadExt: Read {
    fn read_retry(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut read = 0;
        while read != buffer.len() {
            match self.read(&mut buffer[read..]) {
                Ok(0) => break,
                Ok(r) => read += r,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(read)
    }
}

impl<T: Read> ReadExt for T {}

pub trait FileExt {
    fn read_offset_retry(&self, buffer: &mut [u8], offset: u64) -> io::Result<usize>;
}

impl FileExt for File {
    fn read_offset_retry(&self, buffer: &mut [u8], offset: u64) -> io::Result<usize> {
        // Make sure the additions in this function don't overflow
        let _end_offset = offset
            .checked_add(buffer.len().u64())
            .ok_or_else(seek_overflow)?;

        let mut read = 0;
        while read != buffer.len() {
            match self.read_offset(&mut buffer[read..], offset + read.u64()) {
                Ok(0) => break,
                Ok(r) => read += r,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(read)
    }
}
