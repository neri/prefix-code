//! Canonical Prefix Coder
use crate::bits::{AnyBitValue, BitSize16, BitSize32, BitStreamReader};
use crate::stats::*;
use crate::*;
use crate::{DecodeError, EncodeError};
use core::{cmp, fmt};

pub struct CanonicalPrefixCoder;

impl CanonicalPrefixCoder {
    /// Repeat the previous value `3 + readbits(2)` times
    pub const REP3P2: u8 = 16;
    /// Repeat 0 `3 + readbits(3)` times
    pub const REP3Z3: u8 = 17;
    /// Repeat 0 `11 + readbits(7)` times
    pub const REP11Z7: u8 = 18;

    pub const PREFIX_PERMUTATION_ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1,
        15,
        // 17, 18, 0, 1, 2, 3, 4, 5, 16, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];

    pub fn generate_prefix_table_with<K>(
        max_size: BitSize16,
        mut iter: impl Iterator<Item = K>,
    ) -> Vec<Option<AnyBitValue>>
    where
        K: Copy + Ord + Into<usize>,
        // K: fmt::Debug + fmt::Display + fmt::LowerHex,
    {
        let mut freq_table = BTreeMap::new();
        while let Some(key) = iter.next() {
            freq_table.count_freq(key);
        }
        let freq_table = freq_table.into_freq_table(true);
        let prefix_table = CanonicalPrefixCoder::generate_prefix_table(&freq_table, max_size);
        let max_symbol = prefix_table.iter().fold(0usize, |a, v| a.max((v.0).into()));
        let mut prefix_map = Vec::new();
        prefix_map.resize(1 + max_symbol, None);
        for item in prefix_table.iter() {
            prefix_map[(item.0).into()] = Some(item.1);
        }
        prefix_map
    }

    pub fn generate_prefix_table<K>(
        freq_table: &[(K, usize)],
        max_size: BitSize16,
    ) -> Vec<(K, AnyBitValue)>
    where
        K: Copy + Ord,
    {
        if freq_table.len() <= 2 {
            let mut input = freq_table.to_vec();
            input.sort_by(|a, b| a.0.cmp(&b.0));
            let mut result = Vec::new();
            for (index, item) in input.iter().enumerate() {
                result.push((item.0, AnyBitValue::new(BitSize32::Bit1, index as u32)));
            }
            return result;
        }

        let mut freq_table = Vec::from_iter(freq_table.iter());
        freq_table.sort_by(|a, b| match b.1.cmp(&a.1) {
            cmp::Ordering::Equal => a.0.cmp(&b.0),
            ord => ord,
        });

        let mut tree = freq_table
            .iter()
            .map(|v| HuffmanTreeNode::make_leaf(v.0, v.1))
            .collect::<Vec<_>>();
        while tree.len() > 1 {
            tree.sort_by(|a, b| a.order(b));
            let left = tree.pop().unwrap();
            let right = tree.pop().unwrap();
            let node = HuffmanTreeNode::make_pair(left, right);
            tree.push(node);
        }

        let mut prefix_size_table = BTreeMap::new();
        tree[0].count_prefix_size(&mut prefix_size_table, 0);
        let actual_max_len = 1 + prefix_size_table.iter().fold(0, |a, v| a.max(*v.0));
        let mut prefix_lengths = Vec::new();
        prefix_lengths.resize(actual_max_len as usize, 0);
        for item in prefix_size_table.into_iter() {
            prefix_lengths[item.0 as usize] = item.1;
        }

        Self::_adjust_prefix_lengths(&mut prefix_lengths, max_size);

        let mut acc = 0;
        let mut last_bits = 0;
        let mut prefix_codes = Vec::new();
        for (size, len) in prefix_lengths.into_iter().enumerate() {
            for _ in 0..len {
                let mut adj = size;
                while last_bits < adj {
                    acc <<= 1;
                    adj -= 1;
                }
                last_bits = size;
                prefix_codes.push(AnyBitValue::new(BitSize32::new(size as u8).unwrap(), acc));
                acc += 1;
            }
        }

        let mut prefix_table = freq_table
            .iter()
            .zip(prefix_codes.iter())
            .map(|(a, &b)| (a.0, b))
            .collect::<Vec<_>>();
        prefix_table.sort_by(|a, b| match a.1.size().cmp(&b.1.size()) {
            cmp::Ordering::Equal => a.0.cmp(&b.0),
            ord => ord,
        });
        for (p, &q) in prefix_table.iter_mut().zip(prefix_codes.iter()) {
            p.1 = q;
        }

        prefix_table
    }

    fn _adjust_prefix_lengths(prefix_size_table: &mut [usize], max_size: BitSize16) {
        let max_size = max_size as usize;
        if prefix_size_table.len() <= max_size {
            return;
        }
        let mut extra_bits = 0;
        for item in prefix_size_table.iter_mut().skip(max_size) {
            extra_bits += *item;
            *item = 0;
        }
        prefix_size_table[max_size] += extra_bits;

        let mut total = 0;
        for i in (1..=max_size).rev() {
            total += prefix_size_table[i] << (max_size - i);
        }

        let one = 1usize << max_size;
        while total > one {
            prefix_size_table[max_size] -= 1;

            for i in (1..=max_size - 1).rev() {
                if prefix_size_table[i] > 0 {
                    prefix_size_table[i] -= 1;
                    prefix_size_table[i + 1] += 2;
                    break;
                }
            }

            total -= 1;
        }
    }

    fn rle_match_len(value: u8, data: &[u8], cursor: usize, max_len: usize) -> usize {
        unsafe {
            let max_len = (data.len() - cursor).min(max_len);
            let p = data.as_ptr().add(cursor);
            for len in 0..max_len {
                if p.add(len).read_volatile() != value {
                    return len;
                }
            }
            max_len
        }
    }

    fn rle_compress_prefix_table(input: &[u8]) -> Vec<AnyBitValue> {
        let mut output = Vec::new();
        let mut cursor = 0;
        let mut prev = 8;
        while let Some(current) = input.get(cursor) {
            let current = *current;
            cursor += {
                if current > 0 {
                    if current == prev {
                        let len = Self::rle_match_len(prev, &input, cursor, 6);
                        if len >= 3 {
                            output.push(AnyBitValue::with_byte(Self::REP3P2));
                            output.push(AnyBitValue::new(BitSize32::Bit2, len as u32 - 3));
                            len
                        } else {
                            output.push(AnyBitValue::with_byte(current));
                            1
                        }
                    } else {
                        output.push(AnyBitValue::with_byte(current));
                        prev = current;
                        1
                    }
                } else {
                    let len = Self::rle_match_len(0, &input, cursor, 138);
                    if len >= 11 {
                        output.push(AnyBitValue::with_byte(Self::REP11Z7));
                        output.push(AnyBitValue::new(BitSize32::Bit7, len as u32 - 11));
                        len
                    } else if len >= 3 {
                        output.push(AnyBitValue::with_byte(Self::REP3Z3));
                        output.push(AnyBitValue::new(BitSize32::Bit3, len as u32 - 3));
                        len
                    } else {
                        output.push(AnyBitValue::with_byte(current));
                        1
                    }
                }
            };
        }
        output
    }

    pub fn encode_single_prefix_table(
        input: &[Option<AnyBitValue>],
    ) -> Result<MetaPrefixTable, EncodeError> {
        let table0 = input
            .iter()
            .map(|v| match v {
                Some(v) => v.size().as_u8(),
                None => 0,
            })
            .collect::<Vec<_>>();
        Self::encode_prefix_tables(&[&table0])
    }

    pub fn encode_prefix_tables(tables: &[&[u8]]) -> Result<MetaPrefixTable, EncodeError> {
        let hlits = tables.iter().map(|v| v.len()).collect::<Vec<_>>();

        let tables = tables
            .iter()
            .map(|v| Self::rle_compress_prefix_table(v))
            .collect::<Vec<_>>();

        let mut freq_table = BTreeMap::new();
        for table in tables.iter() {
            for bits in table.iter() {
                if bits.size == BitSize32::OCTET {
                    freq_table.count_freq(bits.value())
                }
            }
        }
        let freq_table = freq_table.into_freq_table(true);

        let prefix_table =
            CanonicalPrefixCoder::generate_prefix_table(&freq_table, BitSize16::Bit7);
        let mut prefix_map = [None; 20];
        for prefix in prefix_table.iter() {
            assert!(prefix.1.size() < BitSize32::OCTET);
            prefix_map[prefix.0 as usize] = Some(prefix.1);
        }

        let mut compressed_table = Vec::new();
        for table in tables.iter() {
            for &item in table.iter() {
                if item.size == BitSize32::OCTET {
                    let prefix_code = prefix_map[item.value as usize].unwrap();
                    compressed_table.push(prefix_code.reversed());
                } else {
                    compressed_table.push(item);
                }
            }
        }

        let mut prefix_sizes = [None; 19];
        let mut max_index = 3;
        for (p, q) in Self::PREFIX_PERMUTATION_ORDER.into_iter().enumerate() {
            if let Some(item) = prefix_map[q] {
                max_index = max_index.max(p);
                prefix_sizes[p] = Some(item.size);
            }
        }
        let mut prefix_table = Vec::new();
        for &item in prefix_sizes.iter().take(1 + max_index) {
            prefix_table.push(AnyBitValue::new(
                BitSize32::Bit3,
                item.map(|v| v as u32).unwrap_or_default(),
            ));
        }

        Ok(MetaPrefixTable {
            hlits,
            hclen: max_index - 3,
            prefix_table,
            payload: compressed_table,
        })
    }

    pub fn decode_prefix_table_from_bytes(
        bytes: &[u8],
        output_size: usize,
    ) -> Result<Vec<u8>, DecodeError> {
        let mut reader = BitStreamReader::new(bytes);
        let mut output = Vec::<u8>::new();
        Self::decode_prefix_tables(&mut reader, &mut output, &[output_size])?;
        Ok(output)
    }

    pub fn decode_prefix_tables<'a>(
        reader: &mut BitStreamReader<'a>,
        output: &mut Vec<u8>,
        output_sizes: &[usize],
    ) -> Result<(), DecodeError> {
        output
            .try_reserve_exact(output_sizes.iter().fold(0, |a, v| a + v))
            .map_err(|_| DecodeError::OutOfMemory)?;

        let num_prefixes = 4 + reader.read_nibble().ok_or(DecodeError::InvalidData)? as usize;
        let mut prefixes = Vec::new();
        for index in Self::PREFIX_PERMUTATION_ORDER
            .into_iter()
            .take(num_prefixes)
        {
            let prefix_bit = reader
                .read(BitSize32::Bit3)
                .ok_or(DecodeError::InvalidData)?;
            prefixes.push((index as u8, prefix_bit as u8));
        }

        let decoder = CanonicalPrefixDecoder::new(&prefixes);
        let mut limit = 0;
        for size in output_sizes {
            limit += size;
            let mut prev = 8;
            while output.len() < limit {
                let decoded = decoder.decode(reader)? as u8;
                match decoded {
                    0 => {
                        output.push(decoded);
                    }
                    1..=15 => {
                        output.push(decoded);
                        prev = decoded;
                    }
                    Self::REP3P2 => {
                        let ext_bits = 3 + reader
                            .read(BitSize32::Bit2)
                            .ok_or(DecodeError::InvalidData)?;
                        for _ in 0..ext_bits {
                            output.push(prev);
                        }
                    }
                    Self::REP3Z3 => {
                        let ext_bits = 3 + reader
                            .read(BitSize32::Bit3)
                            .ok_or(DecodeError::InvalidData)?;
                        for _ in 0..ext_bits {
                            output.push(0);
                        }
                    }
                    Self::REP11Z7 => {
                        let ext_bits = 11
                            + reader
                                .read(BitSize32::Bit7)
                                .ok_or(DecodeError::InvalidData)?;
                        for _ in 0..ext_bits {
                            output.push(0);
                        }
                    }
                    _ => return Err(DecodeError::InvalidData),
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MetaPrefixTable {
    pub hlits: Vec<usize>,
    pub hclen: usize,
    pub prefix_table: Vec<AnyBitValue>,
    pub payload: Vec<AnyBitValue>,
}

pub struct CanonicalPrefixDecoder {
    prefix_map: BTreeMap<u32, usize>,
    max_size: u8,
    min_size: u8,
}

impl CanonicalPrefixDecoder {
    #[inline]
    fn _key_value(size: u8, value: u32) -> u32 {
        ((size as u32) << 24) | (value)
    }

    #[inline]
    pub fn new<K>(prefixes: &[(K, u8)]) -> Self
    where
        K: Copy + Ord + Into<usize>,
    {
        let prefix_table = Self::reorder_prefix_table(prefixes);
        Self::with_prefix_table(prefix_table)
    }

    pub fn with_prefix_table(prefix_table: Vec<Option<AnyBitValue>>) -> Self {
        let max_size = prefix_table
            .iter()
            .filter_map(|v| v.as_ref())
            .fold(0, |a, v| a.max(v.size.as_u8()));

        let min_size = prefix_table
            .iter()
            .filter_map(|v| v.as_ref())
            .fold(u8::MAX, |a, v| a.min(v.size.as_u8()));

        let mut prefix_map = BTreeMap::new();
        for (index, item) in prefix_table.iter().enumerate() {
            if let Some(item) = item {
                prefix_map.insert(Self::_key_value(item.size.as_u8(), item.value()), index);
            }
        }

        Self {
            prefix_map,
            max_size,
            min_size,
        }
    }

    pub fn reorder_prefix_table<K>(prefixes: &[(K, u8)]) -> Vec<Option<AnyBitValue>>
    where
        K: Copy + Ord + Into<usize>,
    {
        let mut prefixes = prefixes.to_vec();
        prefixes.sort_by(|a, b| match a.1.cmp(&b.1) {
            cmp::Ordering::Equal => a.0.cmp(&b.0),
            ord => ord,
        });

        let mut acc = 0;
        let mut last_bits = 0;
        let mut prefix_table = Vec::new();

        prefix_table.resize(
            1 + prefixes.iter().fold(0usize, |a, v| a.max(v.0.into())),
            None,
        );

        for item in prefixes.iter() {
            let bits = item.1;
            if bits > 0 {
                let mut adj = bits;
                while last_bits < adj {
                    acc <<= 1;
                    adj -= 1;
                }
                last_bits = bits;
                prefix_table[(item.0).into()] =
                    Some(AnyBitValue::new(BitSize32::new(bits).unwrap(), acc));
                acc += 1;
            }
        }
        prefix_table
    }

    pub fn decode(&self, reader: &mut BitStreamReader) -> Result<usize, DecodeError> {
        let mut read_bits = self.min_size;
        let mut value = 0;
        for _ in 0..self.min_size {
            let read = reader.read_bool().ok_or(DecodeError::InvalidData)?;
            value = (value << 1) | read as u32;
        }
        loop {
            if let Some(decoded) = self.prefix_map.get(&Self::_key_value(read_bits, value)) {
                return Ok(*decoded);
            } else {
                if read_bits >= self.max_size {
                    for item in self.prefix_map.iter() {
                        let size = item.0 >> 24;
                        let value = item.0 & 0xFFFF;
                        let bits = AnyBitValue::new(BitSize32::new(size as u8).unwrap(), value);
                        println!("DECODED {:02x} {:2} {:04x} {}", item.1, size, value, bits);
                    }
                    panic!(
                        "UNKNOWN CHC VALUE {} {:04x} {}",
                        read_bits,
                        value,
                        AnyBitValue::new(BitSize32::new(read_bits).unwrap(), value)
                    );
                    // return Err(DecodeError::InvalidData);
                }
                let read = reader.read_bool().ok_or(DecodeError::InvalidData)?;
                value = (value << 1) | read as u32;
                read_bits += 1;
            }
        }
    }
}

pub struct HuffmanTreeNode<K> {
    data: K,
    freq: usize,
    left: Option<Box<HuffmanTreeNode<K>>>,
    right: Option<Box<HuffmanTreeNode<K>>>,
}

impl<K: Copy> HuffmanTreeNode<K> {
    #[inline]
    pub fn make_leaf(data: K, freq: usize) -> Self {
        Self {
            data,
            freq,
            left: None,
            right: None,
        }
    }

    #[inline]
    pub fn make_pair(left: Self, right: Self) -> Self {
        let freq = left.freq + right.freq;
        Self {
            data: left.data,
            freq,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self.left, None) && matches!(self.right, None)
    }

    fn count_prefix_size(&self, map: &mut BTreeMap<u8, usize>, chc_bit: u8) {
        if let Some(left) = self.left.as_ref() {
            left.count_prefix_size(map, chc_bit + 1);
        }
        if let Some(right) = self.right.as_ref() {
            right.count_prefix_size(map, chc_bit + 1);
        }
        if self.is_leaf() {
            map.entry(chc_bit).and_modify(|v| *v += 1).or_insert(1);
        }
    }

    fn order(&self, other: &Self) -> cmp::Ordering
    where
        K: Ord,
    {
        match other.freq.cmp(&self.freq) {
            cmp::Ordering::Equal => match (self.is_leaf(), other.is_leaf()) {
                (true, true) | (false, false) => other.data.cmp(&self.data),
                (true, false) => cmp::Ordering::Greater,
                (false, true) => cmp::Ordering::Less,
            },
            ord => ord,
        }
    }
}

impl<K: fmt::Debug> fmt::Debug for HuffmanTreeNode<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HuffmanTreeNode")
            .field("data", &self.data)
            .field("freq", &self.freq)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn chc() {
        // Count 	270 	20 	10 	0 	1 	6 	1
        // Huffman 	0 	10 	110 	- 	11110 	1110 	11111
        // maxLength = 4 	0 	10 	1100 	- 	1101 	1110 	1111
        // maxLength = 3 	00 	01 	100 	- 	101 	110 	111
    }
}
