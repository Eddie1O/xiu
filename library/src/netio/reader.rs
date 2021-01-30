use super::errors::{IOReadError, IOReadErrorValue};
use byteorder::{ByteOrder, ReadBytesExt};
use bytes::BytesMut;
use std::io;
use std::io::Cursor;
use std::time::Duration;

use tokio::{prelude::*, stream::StreamExt, time::timeout};
use tokio_util::codec::BytesCodec;
use tokio_util::codec::Framed;

pub struct Reader {
    buffer: BytesMut,
}
impl Reader {
    pub fn new(input: BytesMut) -> Self {
        Self { buffer: input }
    }

    // pub fn new_with_extend(input: BytesMut, extend: &[u8]) -> Reader {
    //     let mut reader = Reader { buffer: input };
    //     reader.extend_from_slice(extend);
    //     reader
    // }

    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        self.buffer.extend_from_slice(extend)
    }
    pub fn read_bytes(&mut self, bytes_num: usize) -> Result<BytesMut, IOReadError> {
        if self.buffer.len() < bytes_num {
            return Err(IOReadError {
                value: IOReadErrorValue::NotEnoughBytes,
            });
        }
        Ok(self.buffer.split_to(bytes_num))
    }

    pub fn advance_bytes(&mut self, bytes_num: usize) -> Result<BytesMut, IOReadError> {
        if self.buffer.len() < bytes_num {
            return Err(IOReadError {
                value: IOReadErrorValue::NotEnoughBytes,
            });
        }

        //here maybe optimised
        Ok(self.buffer.clone().split_to(bytes_num))
    }

    pub fn read_bytes_cursor(&mut self, bytes_num: usize) -> Result<Cursor<BytesMut>, IOReadError> {
        let tmp_bytes = self.read_bytes(bytes_num)?;
        let tmp_cursor = Cursor::new(tmp_bytes);
        Ok(tmp_cursor)
    }

    pub fn advance_bytes_cursor(
        &mut self,
        bytes_num: usize,
    ) -> Result<Cursor<BytesMut>, IOReadError> {
        let tmp_bytes = self.advance_bytes(bytes_num)?;
        let tmp_cursor = Cursor::new(tmp_bytes);
        Ok(tmp_cursor)
    }

    pub fn read_u8(&mut self) -> Result<u8, IOReadError> {
        let mut cursor = self.read_bytes_cursor(1)?;

        Ok(cursor.read_u8()?)
    }

    pub fn advance_u8(&mut self) -> Result<u8, IOReadError> {
        let mut cursor = self.advance_bytes_cursor(1)?;
        Ok(cursor.read_u8()?)
    }

    pub fn read_u16<T: ByteOrder>(&mut self) -> Result<u16, IOReadError> {
        let mut cursor = self.read_bytes_cursor(2)?;
        let val = cursor.read_u16::<T>()?;
        Ok(val)
    }

    pub fn read_u24<T: ByteOrder>(&mut self) -> Result<u32, IOReadError> {
        let mut cursor = self.read_bytes_cursor(3)?;
        let val = cursor.read_u24::<T>()?;
        Ok(val)
    }

    pub fn advance_u24<T: ByteOrder>(&mut self) -> Result<u32, IOReadError> {
        let mut cursor = self.advance_bytes_cursor(3)?;
        Ok(cursor.read_u24::<T>()?)
    }

    pub fn read_u32<T: ByteOrder>(&mut self) -> Result<u32, IOReadError> {
        let mut cursor = self.read_bytes_cursor(4)?;
        let val = cursor.read_u32::<T>()?;

        Ok(val)
    }

    pub fn read_f64<T: ByteOrder>(&mut self) -> Result<f64, IOReadError> {
        let mut cursor = self.read_bytes_cursor(8)?;
        let val = cursor.read_f64::<T>()?;

        Ok(val)
    }
}

pub struct NetworkReader<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub reader: Reader,
    bytes_stream: Framed<S, BytesCodec>,
    timeout: Duration,
}

impl<S> NetworkReader<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S, ms: Duration) -> Self {
        Self {
            reader: Reader::new(BytesMut::new()),
            bytes_stream: Framed::new(stream, BytesCodec::new()),
            timeout: ms,
        }
    }

    pub async fn try_next(&mut self) -> Result<(), IOReadError> {
        let val = self.bytes_stream.try_next();
        match timeout(self.timeout, val).await? {
            Ok(Some(data)) => {
                self.reader.extend_from_slice(&data[..]);
                Ok(())
            }
            _ => Err(IOReadError {
                value: IOReadErrorValue::IO(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "cannot get data",
                )),
            }),
        }
    }
}