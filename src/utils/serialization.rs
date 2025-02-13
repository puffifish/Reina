// File: src/utils/serialization.rs

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::error::Error;
use std::fmt;
use std::io::Cursor;
use std::hint::black_box;
use blake3; // Blake3 leverages SIMD and multithreading
use rayon::prelude::*;

/// Supported endianness.
#[derive(Clone, Copy, Debug)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    /// Writes a u32 value into a buffer using the selected endianness.
    #[inline(always)]
    pub fn write_u32(&self, value: u32, buf: &mut [u8]) -> SerializationResult<usize> {
        if buf.len() < 4 {
            return Err(SerializationError::BufferTooSmall);
        }
        match self {
            Endianness::Little => (&mut buf[..4]).write_u32::<LittleEndian>(value)?,
            Endianness::Big => (&mut buf[..4]).write_u32::<BigEndian>(value)?,
        }
        Ok(4)
    }
    /// Writes a u64 value into a buffer using the selected endianness.
    #[inline(always)]
    pub fn write_u64(&self, value: u64, buf: &mut [u8]) -> SerializationResult<usize> {
        if buf.len() < 8 {
            return Err(SerializationError::BufferTooSmall);
        }
        match self {
            Endianness::Little => (&mut buf[..8]).write_u64::<LittleEndian>(value)?,
            Endianness::Big => (&mut buf[..8]).write_u64::<BigEndian>(value)?,
        }
        Ok(8)
    }
}

/// Custom error type.
#[derive(Debug)]
pub enum SerializationError {
    IoError(std::io::Error),
    ChecksumMismatch { stored: Vec<u8>, computed: Vec<u8> },
    InvalidData(String),
    BufferTooSmall,
    Overflow,
}

impl From<std::io::Error> for SerializationError {
    fn from(err: std::io::Error) -> Self {
        SerializationError::IoError(err)
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::IoError(e) => write!(f, "I/O error: {}", e),
            SerializationError::ChecksumMismatch { stored, computed } => write!(
                f,
                "Checksum mismatch: stored {:?} vs computed {:?}",
                stored, computed
            ),
            SerializationError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            SerializationError::BufferTooSmall => write!(f, "Buffer too small"),
            SerializationError::Overflow => write!(f, "Integer overflow in length calculation"),
        }
    }
}

impl Error for SerializationError {}

pub type SerializationResult<T> = Result<T, SerializationError>;

/// Encode and Decode traits.
pub trait Encode {
    fn encoded_size(&self) -> usize;
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize>;
}

pub trait Decode: Sized {
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)>;
}

/// --- Varint and ZigZag Helper Functions ---
#[inline(always)]
fn encode_varint_u64(mut value: u64, buffer: &mut [u8]) -> SerializationResult<usize> {
    let mut i = 0;
    loop {
        if i >= buffer.len() {
            return Err(SerializationError::BufferTooSmall);
        }
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            buffer[i] = byte;
            i += 1;
            break;
        } else {
            buffer[i] = byte | 0x80;
            i += 1;
        }
    }
    Ok(i)
}

#[inline(always)]
fn decode_varint_u64(buffer: &[u8]) -> SerializationResult<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    let mut i = 0;
    while i < buffer.len() {
        let byte = buffer[i];
        let part = (byte & 0x7F) as u64;
        value |= part.checked_shl(shift).ok_or(SerializationError::Overflow)?;
        i += 1;
        if byte & 0x80 == 0 {
            return Ok((value, i));
        }
        shift += 7;
        if shift >= 64 {
            return Err(SerializationError::InvalidData("varint overflow".into()));
        }
    }
    Err(SerializationError::InvalidData("buffer ended unexpectedly while reading varint".into()))
}

#[inline(always)]
fn encode_varint_u32(value: u32, buffer: &mut [u8]) -> SerializationResult<usize> {
    encode_varint_u64(value as u64, buffer)
}

#[inline(always)]
fn decode_varint_u32(buffer: &[u8]) -> SerializationResult<(u32, usize)> {
    let (value, consumed) = decode_varint_u64(buffer)?;
    if value > u32::MAX as u64 {
        return Err(SerializationError::InvalidData("u32 varint overflow".into()));
    }
    Ok((value as u32, consumed))
}

#[inline(always)]
fn encode_zigzag_i32(value: i32) -> u32 {
    ((value << 1) ^ (value >> 31)) as u32
}

#[inline(always)]
fn decode_zigzag_i32(value: u32) -> i32 {
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

#[inline(always)]
fn encode_zigzag_i64(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

#[inline(always)]
fn decode_zigzag_i64(value: u64) -> i64 {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
}

/// --- Primitive Implementations ---
impl Encode for u64 {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let mut value = *self;
        let mut size = 0;
        while value >= 0x80 {
            size += 1;
            value >>= 7;
        }
        size + 1
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], _endianness: Endianness) -> SerializationResult<usize> {
        encode_varint_u64(*self, buffer)
    }
}

impl Decode for u64 {
    #[inline(always)]
    fn decode_from(buffer: &[u8], _endianness: Endianness) -> SerializationResult<(Self, usize)> {
        decode_varint_u64(buffer)
    }
}

impl Encode for u32 {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let mut value = *self;
        let mut size = 0;
        while value >= 0x80 {
            size += 1;
            value >>= 7;
        }
        size + 1
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], _endianness: Endianness) -> SerializationResult<usize> {
        encode_varint_u32(*self, buffer)
    }
}

impl Decode for u32 {
    #[inline(always)]
    fn decode_from(buffer: &[u8], _endianness: Endianness) -> SerializationResult<(Self, usize)> {
        decode_varint_u32(buffer)
    }
}

impl Encode for i32 {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let zigzag = encode_zigzag_i32(*self);
        zigzag.encoded_size()
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        let zigzag = encode_zigzag_i32(*self);
        zigzag.encode_to(buffer, endianness)
    }
}

impl Decode for i32 {
    #[inline(always)]
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)> {
        let (value, consumed) = u32::decode_from(buffer, endianness)?;
        Ok((decode_zigzag_i32(value), consumed))
    }
}

impl Encode for i64 {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let zigzag = encode_zigzag_i64(*self);
        zigzag.encoded_size()
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        let zigzag = encode_zigzag_i64(*self);
        zigzag.encode_to(buffer, endianness)
    }
}

impl Decode for i64 {
    #[inline(always)]
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)> {
        let (value, consumed) = u64::decode_from(buffer, endianness)?;
        Ok((decode_zigzag_i64(value), consumed))
    }
}

impl Encode for bool {
    #[inline(always)]
    fn encoded_size(&self) -> usize { 1 }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], _endianness: Endianness) -> SerializationResult<usize> {
        if buffer.is_empty() {
            return Err(SerializationError::BufferTooSmall);
        }
        buffer[0] = if *self { 1 } else { 0 };
        Ok(1)
    }
}

impl Decode for bool {
    #[inline(always)]
    fn decode_from(buffer: &[u8], _endianness: Endianness) -> SerializationResult<(Self, usize)> {
        if buffer.is_empty() {
            return Err(SerializationError::InvalidData("Empty buffer when expecting bool".into()));
        }
        match buffer[0] {
            0 => Ok((false, 1)),
            1 => Ok((true, 1)),
            other => Err(SerializationError::InvalidData(format!("Invalid bool value: {}", other))),
        }
    }
}

impl Encode for f64 {
    #[inline(always)]
    fn encoded_size(&self) -> usize { 8 }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        if buffer.len() < 8 {
            return Err(SerializationError::BufferTooSmall);
        }
        match endianness {
            Endianness::Little => (&mut buffer[..8]).write_f64::<LittleEndian>(*self)?,
            Endianness::Big => (&mut buffer[..8]).write_f64::<BigEndian>(*self)?,
        }
        Ok(8)
    }
}

impl Decode for f64 {
    #[inline(always)]
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)> {
        if buffer.len() < 8 {
            return Err(SerializationError::InvalidData("Buffer too small for f64".into()));
        }
        let value = match endianness {
            Endianness::Little => {
                let mut rdr = Cursor::new(&buffer[..8]);
                rdr.read_f64::<LittleEndian>()?
            },
            Endianness::Big => {
                let mut rdr = Cursor::new(&buffer[..8]);
                rdr.read_f64::<BigEndian>()?
            },
        };
        Ok((value, 8))
    }
}

impl Encode for String {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let len = self.as_bytes().len();
        let mut size = 0;
        let mut temp = len as u64;
        while temp >= 0x80 { size += 1; temp >>= 7; }
        size + 1 + len
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], _endianness: Endianness) -> SerializationResult<usize> {
        let bytes = self.as_bytes();
        let len = bytes.len();
        let mut varint_size = 0;
        let mut temp = len as u64;
        while temp >= 0x80 { varint_size += 1; temp >>= 7; }
        varint_size += 1;
        if buffer.len() < varint_size + len {
            return Err(SerializationError::BufferTooSmall);
        }
        let written = encode_varint_u64(len as u64, buffer)?;
        buffer[written..written+len].copy_from_slice(bytes);
        Ok(written + len)
    }
}

impl Decode for String {
    #[inline(always)]
    fn decode_from(buffer: &[u8], _endianness: Endianness) -> SerializationResult<(Self, usize)> {
        let (len, varint_size) = decode_varint_u64(buffer)?;
        let total = varint_size.checked_add(len as usize).ok_or(SerializationError::Overflow)?;
        if buffer.len() < total {
            return Err(SerializationError::InvalidData("Not enough bytes for String".into()));
        }
        let string_bytes = &buffer[varint_size..total];
        match std::str::from_utf8(string_bytes) {
            Ok(s) => Ok((s.to_owned(), total)),
            Err(e) => Err(SerializationError::InvalidData(format!("UTF-8 error: {:?}", e))),
        }
    }
}

impl Encode for Vec<u8> {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        let len = self.len();
        let mut size = 0;
        let mut temp = len as u64;
        while temp >= 0x80 { size += 1; temp >>= 7; }
        size + 1 + len
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], _endianness: Endianness) -> SerializationResult<usize> {
        let len = self.len();
        let mut varint_size = 0;
        let mut temp = len as u64;
        while temp >= 0x80 { varint_size += 1; temp >>= 7; }
        varint_size += 1;
        if buffer.len() < varint_size + len {
            return Err(SerializationError::BufferTooSmall);
        }
        let written = encode_varint_u64(len as u64, buffer)?;
        buffer[written..written+len].copy_from_slice(self);
        Ok(written + len)
    }
}

impl Decode for Vec<u8> {
    #[inline(always)]
    fn decode_from(buffer: &[u8], _endianness: Endianness) -> SerializationResult<(Self, usize)> {
        let (len, varint_size) = decode_varint_u64(buffer)?;
        let total = varint_size.checked_add(len as usize).ok_or(SerializationError::Overflow)?;
        if buffer.len() < total {
            return Err(SerializationError::InvalidData("Not enough bytes for Vec<u8>".into()));
        }
        let bytes = buffer[varint_size..total].to_vec();
        Ok((bytes, total))
    }
}

/// --- Transaction Struct ---
/// Fields reordered for improved alignment.
#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub id: u64,
    pub amount: u64,
    pub fee: f64,
    pub version: u8,
    pub sender: String,
    pub recipient: String,
    pub signature: Vec<u8>,
}

impl Encode for Transaction {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        self.id.encoded_size() +
        self.amount.encoded_size() +
        self.fee.encoded_size() +
        1 + // version
        self.sender.encoded_size() +
        self.recipient.encoded_size() +
        self.signature.encoded_size()
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        let mut offset = 0;
        offset += self.id.encode_to(&mut buffer[offset..], endianness)?;
        offset += self.amount.encode_to(&mut buffer[offset..], endianness)?;
        offset += self.fee.encode_to(&mut buffer[offset..], endianness)?;
        if buffer.len() < offset + 1 { return Err(SerializationError::BufferTooSmall); }
        buffer[offset] = self.version;
        offset += 1;
        offset += self.sender.encode_to(&mut buffer[offset..], endianness)?;
        offset += self.recipient.encode_to(&mut buffer[offset..], endianness)?;
        offset += self.signature.encode_to(&mut buffer[offset..], endianness)?;
        Ok(offset)
    }
}

impl Decode for Transaction {
    #[inline(always)]
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)> {
        let mut offset = 0;
        let (id, consumed) = u64::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (amount, consumed) = u64::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (fee, consumed) = f64::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        if buffer.len() < offset + 1 { return Err(SerializationError::BufferTooSmall); }
        let version = buffer[offset];
        offset += 1;
        let (sender, consumed) = String::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (recipient, consumed) = String::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (signature, consumed) = Vec::<u8>::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        Ok((Transaction { id, amount, fee, version, sender, recipient, signature }, offset))
    }
}

/// --- Block Struct ---
#[derive(Debug, PartialEq)]
pub struct Block {
    pub version: u8,
    pub block_number: u64,
    pub previous_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
}

impl Encode for Block {
    #[inline(always)]
    fn encoded_size(&self) -> usize {
        1 + self.block_number.encoded_size() +
        self.previous_hash.encoded_size() +
        {
            let mut size = 0;
            let count = self.transactions.len();
            let mut temp = count as u64;
            while temp >= 0x80 { size += 1; temp >>= 7; }
            size + 1 + self.transactions.iter().map(|tx| tx.encoded_size()).sum::<usize>()
        }
    }
    #[inline(always)]
    fn encode_to(&self, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        let mut offset = 0;
        if buffer.is_empty() { return Err(SerializationError::BufferTooSmall); }
        buffer[0] = self.version;
        offset += 1;
        offset += self.block_number.encode_to(&mut buffer[offset..], endianness)?;
        offset += self.previous_hash.encode_to(&mut buffer[offset..], endianness)?;
        let tx_count = self.transactions.len() as u64;
        offset += encode_varint_u64(tx_count, &mut buffer[offset..])?;
        for tx in &self.transactions {
            offset += tx.encode_to(&mut buffer[offset..], endianness)?;
        }
        Ok(offset)
    }
}

impl Decode for Block {
    #[inline(always)]
    fn decode_from(buffer: &[u8], endianness: Endianness) -> SerializationResult<(Self, usize)> {
        if buffer.is_empty() {
            return Err(SerializationError::InvalidData("Empty buffer for Block".into()));
        }
        let version = buffer[0];
        let mut offset = 1;
        let (block_number, consumed) = u64::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (previous_hash, consumed) = Vec::<u8>::decode_from(&buffer[offset..], endianness)?;
        offset += consumed;
        let (tx_count, consumed) = decode_varint_u64(&buffer[offset..])?;
        offset += consumed;
        let mut transactions = Vec::with_capacity(tx_count as usize);
        for _ in 0..tx_count {
            let (tx, consumed) = Transaction::decode_from(&buffer[offset..], endianness)?;
            offset += consumed;
            transactions.push(tx);
        }
        Ok((Block { version, block_number, previous_hash, transactions }, offset))
    }
}

/// --- Serializer with Length Prefix & Checksum ---
/// Format: [length: u32][payload][Blake3 checksum (32 bytes)]
pub struct Serializer;

impl Serializer {
    #[inline(always)]
    fn compute_hash(payload: &[u8]) -> blake3::Hash {
        blake3::hash(payload)
    }

    #[inline(always)]
    pub fn serialize<T: Encode>(data: &T, endianness: Endianness) -> SerializationResult<Vec<u8>> {
        let payload_size = data.encoded_size();
        let total_size = 4_usize
            .checked_add(payload_size)
            .and_then(|v| v.checked_add(32))
            .ok_or(SerializationError::Overflow)?;
        let mut buffer = vec![0u8; total_size];
        let offset = 4;
        let written = data.encode_to(&mut buffer[offset..offset+payload_size], endianness)?;
        if written != payload_size {
            return Err(SerializationError::InvalidData("Encoded size mismatch".into()));
        }
        let payload = &buffer[4..4+payload_size];
        let hash = Self::compute_hash(payload);
        buffer[4+payload_size..].copy_from_slice(hash.as_bytes());
        match endianness {
            Endianness::Little => {
                (&mut buffer[..4]).write_u32::<LittleEndian>((payload_size + 32) as u32)?;
            },
            Endianness::Big => {
                (&mut buffer[..4]).write_u32::<BigEndian>((payload_size + 32) as u32)?;
            },
        }
        Ok(buffer)
    }

    #[inline(always)]
    pub fn deserialize<T: Decode>(buffer: &[u8], endianness: Endianness) -> SerializationResult<T> {
        if buffer.len() < 4 {
            return Err(SerializationError::InvalidData("Buffer too small for length prefix".into()));
        }
        let mut cursor = Cursor::new(&buffer[..4]);
        let len_prefix = match endianness {
            Endianness::Little => cursor.read_u32::<LittleEndian>()?,
            Endianness::Big => cursor.read_u32::<BigEndian>()?,
        } as usize;
        if buffer.len() != 4 + len_prefix {
            return Err(SerializationError::InvalidData("Length prefix does not match buffer size".into()));
        }
        if len_prefix < 32 {
            return Err(SerializationError::InvalidData("Payload length too small to contain checksum".into()));
        }
        let payload_end = 4 + len_prefix - 32;
        let payload = &buffer[4..payload_end];
        let stored_checksum = &buffer[payload_end..4+len_prefix];
        let computed_hash = Self::compute_hash(payload);
        if stored_checksum != computed_hash.as_bytes() {
            return Err(SerializationError::ChecksumMismatch {
                stored: stored_checksum.to_vec(),
                computed: computed_hash.as_bytes().to_vec(),
            });
        }
        let (value, consumed) = T::decode_from(payload, endianness)?;
        if consumed != payload.len() {
            return Err(SerializationError::InvalidData("Extra bytes found in payload after decoding".into()));
        }
        Ok(value)
    }

    // --- Batch Serialization ---
    /// Serializes a slice of items into one contiguous buffer.
    /// Precomputes total buffer size to avoid per–item allocations.
    #[inline(always)]
    pub fn serialize_batch<T: Encode>(data: &[T], endianness: Endianness) -> SerializationResult<Vec<u8>> {
        let total_payload: usize = data.iter().map(|item| item.encoded_size()).sum();
        let mut payload = Vec::with_capacity(total_payload);
        for item in data {
            // Allocate a temporary buffer for each item (minimized by precomputing size)
            let mut temp = vec![0u8; item.encoded_size()];
            let written = item.encode_to(&mut temp, endianness)?;
            payload.extend_from_slice(&temp[..written]);
        }
        let hash = blake3::hash(&payload);
        payload.extend_from_slice(hash.as_bytes());
        let total_length = payload.len();
        let mut output = Vec::with_capacity(4 + total_length);
        output.write_u32::<LittleEndian>(total_length as u32)?;
        output.extend_from_slice(&payload);
        Ok(output)
    }

    // --- Deserialization with Preallocated Buffer ---
    /// For inputs ≤ 4096 bytes, copies the data into a fixed-size stack buffer and calls deserialize().
    #[inline(always)]
    pub fn deserialize_with_pool<T: Decode>(data: &[u8], endianness: Endianness) -> SerializationResult<T> {
        if data.len() <= 4096 {
            let mut stack_buf = [0u8; 4096];
            stack_buf[..data.len()].copy_from_slice(data);
            // Call the full deserialize() so that header and checksum are parsed.
            Serializer::deserialize(&stack_buf[..data.len()], endianness)
        } else {
            Serializer::deserialize(data, endianness)
        }
    }

    // --- Fixed Serialization ---
    /// Uses a fixed-size (121 bytes) buffer for ultra–low–latency serialization.
    const ULTRA_TX_SIZE: usize = 8 + 8 + 8 + 1 + 16 + 16 + 64; // = 121 bytes

    #[inline(always)]
    pub fn serialize_ultra_fixed(tx: &Transaction, endianness: Endianness) -> SerializationResult<[u8; Self::ULTRA_TX_SIZE]> {
        let mut buf = [0u8; Self::ULTRA_TX_SIZE];
        let mut offset = 0;
        // Write id (8 bytes)
        endianness.write_u64(tx.id, &mut buf[offset..offset+8])?;
        offset += 8;
        // Write amount (8 bytes)
        endianness.write_u64(tx.amount, &mut buf[offset..offset+8])?;
        offset += 8;
        // Write fee (8 bytes as f64)
        match endianness {
            Endianness::Little => (&mut buf[offset..offset+8]).write_f64::<LittleEndian>(tx.fee)?,
            Endianness::Big => (&mut buf[offset..offset+8]).write_f64::<BigEndian>(tx.fee)?,
        }
        offset += 8;
        // Write version (1 byte)
        if buf.len() < offset + 1 { return Err(SerializationError::BufferTooSmall); }
        buf[offset] = tx.version;
        offset += 1;
        // Write sender: fixed 16 bytes (padded with zeros)
        let sender_bytes = tx.sender.as_bytes();
        let sender_len = if sender_bytes.len() > 16 { 16 } else { sender_bytes.len() };
        buf[offset..offset+sender_len].copy_from_slice(&sender_bytes[..sender_len]);
        offset += 16;
        // Write recipient: fixed 16 bytes.
        let recipient_bytes = tx.recipient.as_bytes();
        let recipient_len = if recipient_bytes.len() > 16 { 16 } else { recipient_bytes.len() };
        buf[offset..offset+recipient_len].copy_from_slice(&recipient_bytes[..recipient_len]);
        offset += 16;
        // Write signature: fixed 64 bytes.
        let sig_bytes = tx.signature.as_slice();
        let sig_len = if sig_bytes.len() > 64 { 64 } else { sig_bytes.len() };
        buf[offset..offset+sig_len].copy_from_slice(&sig_bytes[..sig_len]);
        offset += 64;
        if offset != Self::ULTRA_TX_SIZE {
            return Err(SerializationError::InvalidData("Ultra TX size mismatch on serialization".into()));
        }
        Ok(buf)
    }

    #[inline(always)]
    pub fn deserialize_ultra_fixed(buf: &[u8; Self::ULTRA_TX_SIZE], endianness: Endianness) -> SerializationResult<Transaction> {
        let mut offset = 0;
        let id = {
            let slice = &buf[offset..offset+8];
            let mut rdr = Cursor::new(slice);
            match endianness {
                Endianness::Little => rdr.read_u64::<LittleEndian>()?,
                Endianness::Big => rdr.read_u64::<BigEndian>()?,
            }
        };
        offset += 8;
        let amount = {
            let slice = &buf[offset..offset+8];
            let mut rdr = Cursor::new(slice);
            match endianness {
                Endianness::Little => rdr.read_u64::<LittleEndian>()?,
                Endianness::Big => rdr.read_u64::<BigEndian>()?,
            }
        };
        offset += 8;
        let fee = {
            let slice = &buf[offset..offset+8];
            let mut rdr = Cursor::new(slice);
            match endianness {
                Endianness::Little => rdr.read_f64::<LittleEndian>()?,
                Endianness::Big => rdr.read_f64::<BigEndian>()?,
            }
        };
        offset += 8;
        if buf.len() < offset + 1 { return Err(SerializationError::BufferTooSmall); }
        let version = buf[offset];
        offset += 1;
        let sender_bytes = &buf[offset..offset+16];
        let sender = String::from_utf8(sender_bytes.iter().cloned().take_while(|&b| b != 0).collect())
            .map_err(|e| SerializationError::InvalidData(format!("Sender UTF-8 error: {}", e)))?;
        offset += 16;
        let recipient_bytes = &buf[offset..offset+16];
        let recipient = String::from_utf8(recipient_bytes.iter().cloned().take_while(|&b| b != 0).collect())
            .map_err(|e| SerializationError::InvalidData(format!("Recipient UTF-8 error: {}", e)))?;
        offset += 16;
        let signature = buf[offset..offset+64].to_vec();
        offset += 64;
        if offset != Self::ULTRA_TX_SIZE {
            return Err(SerializationError::InvalidData("Ultra TX size mismatch on deserialization".into()));
        }
        Ok(Transaction { id, amount, fee, version, sender, recipient, signature })
    }

    /// --- Parallel Deserialization ---
    /// Uses par_chunks_exact(512) for even workload distribution.
    #[inline(always)]
    pub fn parallel_deserialize<T: Decode + Send + 'static>(
        batches: &[Vec<u8>],
        endianness: Endianness,
    ) -> SerializationResult<Vec<T>> {
        let results: Vec<T> = batches.par_chunks_exact(512)
            .flat_map(|chunk| {
                chunk.iter().map(|data| {
                    Serializer::deserialize::<T>(black_box(data), endianness)
                        .expect("Deserialization failed")
                }).collect::<Vec<T>>()
            })
            .collect();
        Ok(results)
    }
}

/// --- Fixed-Length Encoding Utilities ---
pub mod fixed_encoding {
    use super::*;
    #[inline(always)]
    pub fn encode_fixed_u64(value: u64, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        if buffer.len() < 8 { return Err(SerializationError::BufferTooSmall); }
        endianness.write_u64(value, buffer)
    }
    
    #[inline(always)]
    pub fn decode_fixed_u64(buffer: &[u8], endianness: Endianness) -> SerializationResult<(u64, usize)> {
        if buffer.len() < 8 { return Err(SerializationError::InvalidData("Buffer too small for fixed u64".into())); }
        let value = match endianness {
            Endianness::Little => {
                let mut rdr = Cursor::new(&buffer[..8]);
                rdr.read_u64::<LittleEndian>()?
            },
            Endianness::Big => {
                let mut rdr = Cursor::new(&buffer[..8]);
                rdr.read_u64::<BigEndian>()?
            },
        };
        Ok((value, 8))
    }
    
    #[inline(always)]
    pub fn encode_fixed_u32(value: u32, buffer: &mut [u8], endianness: Endianness) -> SerializationResult<usize> {
        if buffer.len() < 4 { return Err(SerializationError::BufferTooSmall); }
        endianness.write_u32(value, buffer)
    }
    
    #[inline(always)]
    pub fn decode_fixed_u32(buffer: &[u8], endianness: Endianness) -> SerializationResult<(u32, usize)> {
        if buffer.len() < 4 { return Err(SerializationError::InvalidData("Buffer too small for fixed u32".into())); }
        let value = match endianness {
            Endianness::Little => {
                let mut rdr = Cursor::new(&buffer[..4]);
                rdr.read_u32::<LittleEndian>()?
            },
            Endianness::Big => {
                let mut rdr = Cursor::new(&buffer[..4]);
                rdr.read_u32::<BigEndian>()?
            },
        };
        Ok((value, 4))
    }
    // Additional functions for i32, i64 can be added if needed.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_u64() -> SerializationResult<()> {
        let mut buf = [0u8; 10];
        let value: u64 = 300;
        let size = encode_varint_u64(value, &mut buf)?;
        let (decoded, consumed) = decode_varint_u64(&buf)?;
        assert_eq!(value, decoded);
        assert_eq!(size, consumed);
        Ok(())
    }

    #[test]
    fn test_primitive_encoding() -> SerializationResult<()> {
        let mut buf = [0u8; 16];
        let val_u32: u32 = 150;
        let written = val_u32.encode_to(&mut buf, Endianness::Little)?;
        let (decoded, consumed) = u32::decode_from(&buf, Endianness::Little)?;
        assert_eq!(val_u32, decoded);
        assert_eq!(written, consumed);
        Ok(())
    }

    #[test]
    fn test_string_encoding() -> SerializationResult<()> {
        let s = String::from("Hello, Blockchain!");
        let size = s.encoded_size();
        let mut buf = vec![0u8; size];
        let written = s.encode_to(&mut buf, Endianness::Little)?;
        let (decoded, consumed) = String::decode_from(&buf, Endianness::Little)?;
        assert_eq!(s, decoded);
        assert_eq!(written, consumed);
        Ok(())
    }

    #[test]
    fn test_transaction_serialization() -> SerializationResult<()> {
        let tx = Transaction {
            id: 42,
            amount: 1000,
            fee: 0.01,
            version: 1,
            sender: "Alice".into(),
            recipient: "Bob".into(),
            signature: vec![1, 2, 3, 4],
        };
        let ser = Serializer::serialize(&tx, Endianness::Little)?;
        let de: Transaction = Serializer::deserialize(&ser, Endianness::Little)?;
        assert_eq!(tx, de);
        Ok(())
    }

    #[test]
    fn test_block_serialization() -> SerializationResult<()> {
        let tx1 = Transaction {
            id: 1,
            amount: 500,
            fee: 0.02,
            version: 1,
            sender: "Alice".into(),
            recipient: "Bob".into(),
            signature: vec![1, 2, 3],
        };
        let tx2 = Transaction {
            id: 2,
            amount: 750,
            fee: 0.03,
            version: 1,
            sender: "Charlie".into(),
            recipient: "Dave".into(),
            signature: vec![4, 5, 6],
        };
        let block = Block {
            version: 1,
            block_number: 10,
            previous_hash: vec![0xde, 0xad, 0xbe, 0xef],
            transactions: vec![tx1, tx2],
        };
        let ser = Serializer::serialize(&block, Endianness::Little)?;
        let de: Block = Serializer::deserialize(&ser, Endianness::Little)?;
        assert_eq!(block, de);
        Ok(())
    }

    #[test]
    fn test_ultra_fixed_serialization() -> SerializationResult<()> {
        let tx = Transaction {
            id: 123456789,
            amount: 5000,
            fee: 0.05,
            version: 1,
            sender: "Alice".into(),
            recipient: "Bob".into(),
            signature: vec![1, 2, 3, 4],
        };
        let ultra = Serializer::serialize_ultra_fixed(&tx, Endianness::Little)?;
        let tx_decoded = Serializer::deserialize_ultra_fixed(&ultra, Endianness::Little)?;
        assert_eq!(tx.id, tx_decoded.id);
        assert_eq!(tx.amount, tx_decoded.amount);
        assert_eq!(tx.fee, tx_decoded.fee);
        assert_eq!(tx.version, tx_decoded.version);
        assert_eq!(tx.sender, tx_decoded.sender);
        assert_eq!(tx.recipient, tx_decoded.recipient);
        assert_eq!(&tx.signature[..], &tx_decoded.signature[..tx.signature.len()]);
        Ok(())
    }
}