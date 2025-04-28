//! Bit processing utilities
use alloc::vec::Vec;
use core::fmt;
use core::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitSize16 {
    Bit1,
    Bit2,
    Bit3,
    Bit4,
    Bit5,
    Bit6,
    Bit7,
    Bit8,
    Bit9,
    Bit10,
    Bit11,
    Bit12,
    Bit13,
    Bit14,
    Bit15,
    Bit16,
}

impl BitSize16 {
    #[inline]
    pub fn as_usize(&self) -> Option<usize> {
        let v = *self as usize;
        (v > 0).then(|| v)
    }

    #[inline]
    pub fn new(value: usize) -> Option<Self> {
        match value {
            1 => Some(Self::Bit1),
            2 => Some(Self::Bit2),
            3 => Some(Self::Bit3),
            4 => Some(Self::Bit4),
            5 => Some(Self::Bit5),
            6 => Some(Self::Bit6),
            7 => Some(Self::Bit7),
            8 => Some(Self::Bit8),
            9 => Some(Self::Bit9),
            10 => Some(Self::Bit10),
            11 => Some(Self::Bit11),
            12 => Some(Self::Bit12),
            13 => Some(Self::Bit13),
            14 => Some(Self::Bit14),
            15 => Some(Self::Bit15),
            16 => Some(Self::Bit16),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitSize32 {
    Bit1 = 1,
    Bit2,
    Bit3,
    Bit4,
    Bit5,
    Bit6,
    Bit7,
    Bit8,
    Bit9,
    Bit10,
    Bit11,
    Bit12,
    Bit13,
    Bit14,
    Bit15,
    Bit16,
    Bit17,
    Bit18,
    Bit19,
    Bit20,
    Bit21,
    Bit22,
    Bit23,
    Bit24,
    Bit25,
    Bit26,
    Bit27,
    Bit28,
    Bit29,
    Bit30,
    Bit31,
    Bit32,
}

impl BitSize32 {
    pub const BIT: Self = Self::Bit1;

    pub const BYTE: Self = Self::Bit8;

    pub const OCTET: Self = Self::Bit8;

    #[inline]
    pub fn as_usize(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    #[inline]
    pub const fn new(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Bit1),
            2 => Some(Self::Bit2),
            3 => Some(Self::Bit3),
            4 => Some(Self::Bit4),
            5 => Some(Self::Bit5),
            6 => Some(Self::Bit6),
            7 => Some(Self::Bit7),
            8 => Some(Self::Bit8),
            9 => Some(Self::Bit9),
            10 => Some(Self::Bit10),
            11 => Some(Self::Bit11),
            12 => Some(Self::Bit12),
            13 => Some(Self::Bit13),
            14 => Some(Self::Bit14),
            15 => Some(Self::Bit15),
            16 => Some(Self::Bit16),
            17 => Some(Self::Bit17),
            18 => Some(Self::Bit18),
            19 => Some(Self::Bit19),
            20 => Some(Self::Bit20),
            21 => Some(Self::Bit21),
            22 => Some(Self::Bit22),
            23 => Some(Self::Bit23),
            24 => Some(Self::Bit24),
            25 => Some(Self::Bit25),
            26 => Some(Self::Bit26),
            27 => Some(Self::Bit27),
            28 => Some(Self::Bit28),
            29 => Some(Self::Bit29),
            30 => Some(Self::Bit30),
            31 => Some(Self::Bit31),
            _ => None,
        }
    }
}

impl core::fmt::Display for BitSize32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_usize())
    }
}

#[inline]
pub fn number_of_bits(value: usize) -> u8 {
    match value.checked_ilog2() {
        Some(v) => 1 + v as u8,
        None => 1,
    }
}

pub fn count_bits(array: &[u8]) -> usize {
    array.chunks(4).fold(0, |a, v| match v.try_into() {
        Ok(v) => a + u32::from_le_bytes(v).count_ones() as usize,
        Err(_) => a + v.iter().fold(0, |a, v| a + v.count_ones() as usize),
    })
}

/// Returns nearest power of two
///
/// # Panics
///
/// UB on `value > usize::MAX/2`
pub fn nearest_power_of_two(value: usize) -> usize {
    if value == 0 {
        return 0;
    }
    let next = value.next_power_of_two();
    if next == value {
        return next;
    }
    let threshold = (next >> 2).wrapping_mul(3);
    if value >= threshold { next } else { next >> 1 }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyBitValue {
    pub value: u32,
    pub size: BitSize32,
}

impl AnyBitValue {
    #[inline]
    pub fn new(size: BitSize32, value: u32) -> Self {
        Self { size, value }
    }

    #[inline]
    pub fn with_bool(value: bool) -> Self {
        Self::new(BitSize32::Bit1, value as u32)
    }

    #[inline]
    pub fn with_nibble(value: u8) -> Self {
        Self::new(BitSize32::Bit4, value as u32 & 0x0F)
    }

    #[inline]
    pub fn with_byte(value: u8) -> Self {
        Self::new(BitSize32::Bit8, value as u32)
    }

    #[inline]
    pub fn with_word(value: u32) -> Self {
        Self::new(BitSize32::Bit32, value)
    }

    #[inline]
    pub fn size(&self) -> BitSize32 {
        self.size
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.value
    }

    #[inline]
    pub fn total_len<'a, T>(iter: T) -> usize
    where
        T: Iterator<Item = &'a Option<AnyBitValue>>,
    {
        (Self::total_bit_count(iter) + 7) / 8
    }

    #[inline]
    pub fn to_vec<T>(iter: T) -> Vec<u8>
    where
        T: Iterator<Item = AnyBitValue>,
    {
        let mut bs = BitStreamWriter::new();
        for ext_bit in iter {
            bs.push(&ext_bit);
        }
        bs.into_bytes()
    }

    #[inline]
    pub fn into_vec<T>(iter: T) -> Vec<u8>
    where
        T: IntoIterator<Item = AnyBitValue>,
    {
        Self::to_vec(iter.into_iter())
    }

    pub fn reversed(&self) -> Self {
        let mut value = 0;
        let mut input = self.value();
        for _ in 0..self.size().as_usize() {
            value = (value << 1) | (input & 1);
            input >>= 1;
        }
        Self {
            size: self.size,
            value,
        }
    }

    #[inline]
    pub fn reverse(&mut self) {
        self.value = self.reversed().value();
    }

    #[inline]
    pub fn total_bit_count<'a, T>(iter: T) -> usize
    where
        T: Iterator<Item = &'a Option<AnyBitValue>>,
    {
        iter.fold(0, |a, v| match v {
            Some(v) => a + v.size() as usize,
            None => a,
        })
    }
}

impl fmt::Display for AnyBitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            if width > self.size.as_usize() {
                for _ in 0..width - self.size.as_usize() {
                    write!(f, " ")?;
                }
            }
        }
        for i in (0..self.size.as_usize()).rev() {
            let bit = self.value.wrapping_shr(i as u32) & 1;
            write!(f, "{}", bit)?;
        }
        Ok(())
    }
}

pub struct BitStreamWriter {
    buf: Vec<u8>,
    acc: u8,
    bit_position: u8,
}

impl BitStreamWriter {
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: Vec::new(),
            acc: 0,
            bit_position: 0,
        }
    }

    #[inline]
    pub fn bit_count(&self) -> usize {
        self.buf.len() * 8 + self.bit_position as usize
    }

    #[inline]
    pub fn push_bool(&mut self, value: bool) {
        self.push(&AnyBitValue::with_bool(value));
    }

    #[inline]
    pub fn push_byte(&mut self, value: u8) {
        self.push(&AnyBitValue::with_byte(value))
    }

    #[inline]
    pub fn push_slice(&mut self, value: &[AnyBitValue]) {
        for item in value.iter() {
            self.push(item);
        }
    }

    pub fn push(&mut self, value: &AnyBitValue) {
        let lowest_bits = 8 - self.bit_position;
        let lowest_bit_mask = ((1u32 << value.size.as_u8().min(lowest_bits)) - 1) as u8;
        let mut acc = self.acc | ((value.value as u8 & lowest_bit_mask) << self.bit_position);
        let mut remain_bits = value.size.as_u8();
        if self.bit_position + remain_bits >= 8 {
            self.buf.push(acc);
            acc = 0;
            remain_bits -= lowest_bits;
            self.bit_position = 0;

            if remain_bits > 0 {
                let value_mask = (1u32 << value.size.as_usize()) - 1;
                let mut acc32 = (value.value() & value_mask) >> lowest_bits;
                while remain_bits >= 8 {
                    self.buf.push(acc32 as u8);
                    acc32 >>= 8;
                    remain_bits -= 8;
                }
                acc = acc32 as u8;
            }
        }

        assert!(
            remain_bits < 8,
            "BITS < 8 BUT {}, input {:?}",
            remain_bits,
            value
        );
        self.acc = acc;
        self.bit_position += remain_bits;
    }

    fn flush(&mut self) {
        if self.bit_position > 0 {
            self.buf.push(self.acc);
            self.acc = 0;
            self.bit_position = 0;
        }
    }

    #[inline]
    pub fn into_bytes(mut self) -> Vec<u8> {
        self.flush();
        self.buf
    }
}

pub struct BitStreamReader<'a> {
    iter: Iter<'a, u8>,
    acc: u8,
    bit_position: u8,
}

impl<'a> BitStreamReader<'a> {
    #[inline]
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            iter: slice.iter(),
            acc: 0,
            bit_position: 0,
        }
    }
}

impl BitStreamReader<'_> {
    #[inline]
    pub fn read_bool(&mut self) -> Option<bool> {
        self.read(BitSize32::Bit1).map(|v| v != 0)
    }

    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        self.read(BitSize32::Bit8).map(|v| v as u8)
    }

    #[inline]
    pub fn read_nibble(&mut self) -> Option<u8> {
        self.read(BitSize32::Bit4).map(|v| v as u8)
    }

    pub fn read(&mut self, bits: BitSize32) -> Option<u32> {
        let bits = bits.as_u8();
        if (self.bit_position & 7) == 0 {
            self.next_byte_bounds()?;
        }
        let mask = (1u32 << bits) - 1;
        let mut acc32 = (self.acc >> self.bit_position) as u32 & mask;
        if self.bit_position + bits <= 8 {
            self.bit_position += bits;
            return Some(acc32);
        }
        let mut shifter = 8 - self.bit_position;
        let mut remain_bits = bits - shifter;

        loop {
            self.acc = *self.iter.next()?;
            if remain_bits <= 8 {
                let mask = (1u32 << remain_bits) - 1;
                acc32 |= (self.acc as u32 & mask) << shifter;
                self.bit_position = remain_bits;
                return Some(acc32);
            } else {
                acc32 |= (self.acc as u32) << shifter;
                shifter += 8;
                remain_bits -= 8;
            }
        }
    }

    pub fn read_leb(&mut self) -> Option<usize> {
        let leading = self.read_byte()?;
        if leading < 128 {
            return Some(leading as usize);
        }
        let mut acc = (leading & 0x7F) as usize;
        let mut cursor = 0;
        loop {
            cursor += 7;
            let trail = self.read_byte()? as usize;
            acc |= (trail & 0x7F) << cursor;
            if trail < 128 {
                break Some(acc);
            }
        }
    }

    #[inline]
    pub fn next_byte_bounds(&mut self) -> Option<()> {
        self.acc = *self.iter.next()?;
        self.bit_position = 0;
        Some(())
    }
}

impl Iterator for BitStreamReader<'_> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_bool()
    }
}

/// Bitstream in which the last little-endian value written is first read as a big-endian value
pub struct ReverseBitStreamReader<'a> {
    iter: Iter<'a, u8>,
    acc: u8,
    bit_position: u8,
}

impl<'a> ReverseBitStreamReader<'a> {
    #[inline]
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            iter: slice.iter(),
            acc: 0,
            bit_position: 0,
        }
    }
}

impl ReverseBitStreamReader<'_> {
    pub fn skip_zeros(&mut self) -> Option<()> {
        loop {
            if self.read_bool()? {
                return Some(());
            }
        }
    }

    #[inline]
    pub fn read_bool(&mut self) -> Option<bool> {
        self.read(BitSize32::Bit1).map(|v| v != 0)
    }

    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        self.read(BitSize32::Bit8).map(|v| v as u8)
    }

    #[inline]
    pub fn read_nibble(&mut self) -> Option<u8> {
        self.read(BitSize32::Bit4).map(|v| v as u8)
    }

    pub fn read(&mut self, bits: BitSize32) -> Option<u32> {
        let bits = bits.as_u8();
        if self.bit_position == 0 {
            self.acc = *self.iter.next_back()?;
            self.bit_position = 8;
        }

        if self.bit_position >= bits {
            let shifter = self.bit_position - bits;
            let mask = (1u32 << bits) - 1;
            let result = ((self.acc >> shifter) as u32) & mask;
            self.bit_position -= bits;
            return Some(result);
        }

        let high_mask = (1u32 << self.bit_position) - 1;
        let mut acc32 = (self.acc as u32) & high_mask;
        let mut remain_bits = bits - self.bit_position;
        loop {
            self.acc = *self.iter.next_back()?;
            if remain_bits <= 8 {
                let shifter = 8 - remain_bits;
                let mask = (1u32 << remain_bits) - 1;
                acc32 <<= remain_bits;
                acc32 |= (self.acc as u32 >> shifter) & mask;
                self.bit_position = 8 - remain_bits;
                return Some(acc32);
            } else {
                acc32 <<= 8;
                acc32 |= self.acc as u32;
                remain_bits -= 8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_test() {
        for padding_size in 1..=16 {
            let padding_mask = (1u32 << padding_size) - 1;
            for value_size in 1..=16 {
                let mask = (1u32 << value_size) - 1;
                for pattern in [
                    0x0u32,
                    u32::MAX,
                    0x55555555,
                    0xAAAAAAAA,
                    0x5A5A5A5A,
                    0xA5A5A5A5,
                    0x0F0F0F0F,
                    0xF0F0F0F0,
                    0xE5E5E5E5,
                    1234578,
                    87654321,
                    0xEDB88320,
                    0x04C11DB7,
                ] {
                    let padding_size = BitSize32::new(padding_size).unwrap();
                    let value_size = BitSize32::new(value_size).unwrap();
                    println!("PADDING {padding_size} VALUE {value_size} PATTERN {pattern:08x}");
                    let pattern_n = !pattern & mask;

                    let mut writer = BitStreamWriter::new();
                    writer.push(&AnyBitValue::new(padding_size, 0));
                    writer.push(&AnyBitValue::new(value_size, pattern));
                    writer.push(&AnyBitValue::new(padding_size, u32::MAX));
                    writer.push(&AnyBitValue::new(value_size, pattern_n));
                    writer.push(&AnyBitValue::new(padding_size, 0));
                    writer.push(&AnyBitValue::with_bool(true));
                    let stream = writer.into_bytes();
                    println!("DATA: {:02x?}", &stream);

                    let mut reader = BitStreamReader::new(&stream);
                    assert_eq!(reader.read(padding_size).unwrap(), 0);
                    assert_eq!(reader.read(value_size).unwrap(), pattern & mask);
                    assert_eq!(reader.read(padding_size).unwrap(), padding_mask);
                    assert_eq!(reader.read(value_size).unwrap(), pattern_n & mask);
                    assert_eq!(reader.read(padding_size).unwrap(), 0);

                    let mut reader = ReverseBitStreamReader::new(&stream);
                    reader.skip_zeros().unwrap();
                    assert_eq!(reader.read(padding_size).unwrap(), 0);
                    assert_eq!(reader.read(value_size).unwrap(), pattern_n & mask);
                    assert_eq!(reader.read(padding_size).unwrap(), padding_mask);
                    assert_eq!(reader.read(value_size).unwrap(), pattern & mask);
                    assert_eq!(reader.read(padding_size).unwrap(), 0);
                }
            }
        }
    }

    #[test]
    fn nearest() {
        for (value, expected) in [
            (0usize, 0usize),
            (1, 1),
            (2, 2),
            (3, 4),
            (4, 4),
            (5, 4),
            (6, 8),
            (7, 8),
            (8, 8),
            (9, 8),
            (10, 8),
            (11, 8),
            (12, 16),
            (13, 16),
            (14, 16),
            (16, 16),
            (16, 16),
        ] {
            let test = nearest_power_of_two(value);

            assert_eq!(test, expected);
        }
    }

    #[test]
    fn reverse() {
        for (size, lhs, rhs) in [
            (32, u32::MAX, u32::MAX),
            (32, 0, 0),
            (32, 0xFFFF_0000, 0x0000_FFFF),
            (32, 0x0000_FFFF, 0xFFFF_0000),
            (32, 0xFF00_FF00, 0x00FF_00FF),
            (32, 0x00FF_00FF, 0xFF00_FF00),
            (32, 0xF0F0_F0F0, 0x0F0F_0F0F),
            (32, 0x0F0F_0F0F, 0xF0F0_F0F0),
            (32, 0xCCCC_CCCC, 0x3333_3333),
            (32, 0x3333_3333, 0xCCCC_CCCC),
            (32, 0xAAAA_AAAA, 0x5555_5555),
            (32, 0x5555_5555, 0xAAAA_AAAA),
            (8, 0x55, 0xAA),
            (8, 0xAA, 0x55),
            (8, 0xC0, 0x03),
            (8, 0x03, 0xC0),
            (8, 0xF0, 0x0F),
            (8, 0x0F, 0xF0),
            (16, 0x1234, 0x2C48),
            (32, 0x1234_5678, 0x1E6A_2C48),
        ] {
            let size = BitSize32::new(size).unwrap();
            let lhs = AnyBitValue::new(size, lhs);
            let rhs = AnyBitValue::new(size, rhs);

            assert_eq!(lhs.reversed(), rhs);
            assert_eq!(lhs, rhs.reversed());

            assert_eq!(lhs.reversed().reversed(), lhs);
            assert_eq!(rhs.reversed().reversed(), rhs);
        }
    }
}
