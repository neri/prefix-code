//! Prefix Code Tester

extern crate alloc;

use prefix::{
    CanonicalPrefixCoder, CanonicalPrefixDecoder,
    bits::{AnyBitValue, BitSize, BitStreamReader, BitStreamWriter},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const HLIT: usize = 257;

#[wasm_bindgen]
pub fn encode(input: &[u8]) -> String {
    let mut freq_table = Vec::new();
    freq_table.resize(256, 0usize);
    for &byte in input.iter() {
        freq_table[byte as usize] += 1;
    }
    let freq_table = freq_table
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| (v > 0).then(|| (i, v)))
        .collect::<Vec<_>>();

    let mut prefix_table = Vec::new();
    prefix_table.resize(HLIT, None);
    for item in CanonicalPrefixCoder::generate_prefix_table(&freq_table, BitSize::Bit15) {
        prefix_table[item.0 as usize] = Some(item.1);
    }

    let meta = CanonicalPrefixCoder::encode_single_prefix_table(&prefix_table).unwrap();

    let mut encoded_str = Vec::new();
    let mut encoded_codes = Vec::new();
    let mut output_bits = 0;
    for &byte in input.iter() {
        let code = prefix_table[byte as usize].unwrap();
        encoded_codes.push(code);
        encoded_str.push(format!("{}", code));
        output_bits += code.size() as usize;
    }

    let mut prefix_table2 = prefix_table
        .iter()
        .enumerate()
        .filter_map(|(index, &v)| v.map(|v| (index, v)))
        .collect::<Vec<_>>();
    prefix_table2.sort_by(|a, b| a.0.cmp(&b.0));

    let input_len = input.len() as f64;
    let prefix_table = freq_table
        .iter()
        .zip(prefix_table2.iter())
        .map(|(freq, prefix)| {
            let symbol_char = if freq.0 < 256 {
                match freq.0 as u8 {
                    ch @ 0x20..=0x7e => {
                        format!("\"{}\"", ch as char)
                    }
                    ch => {
                        format!("\"\\x{:02x}\"", ch)
                    }
                }
            } else {
                "(null)".to_owned()
            };
            PrefixTableEntry {
                symbol: freq.0 as usize,
                symbol_char,
                freq: freq.1,
                freq_rate: freq.1 as f64 / input_len,
                len: prefix.1.size().as_u8(),
                code: format!("{}", prefix.1),
            }
        })
        .collect::<Vec<_>>();

    let mut zip = BitStreamWriter::new();
    zip.push(&AnyBitValue::new(BitSize::Bit1, 0b1)); // BFINAL: 1 true
    zip.push(&AnyBitValue::new(BitSize::Bit2, 0b10)); // BTYPE: 10 dynamic huffman
    zip.push(&AnyBitValue::new(BitSize::Bit5, 0)); // HLIT: 0
    zip.push(&AnyBitValue::new(BitSize::Bit5, 0)); // HDIST: 0
    zip.push(&AnyBitValue::new(BitSize::Bit4, meta.hclen as u32)); // HCLEN
    for item in meta.prefix_table.iter() {
        zip.push(item);
    }
    for item in meta.payload.iter() {
        zip.push(item);
    }
    for code in encoded_codes.iter() {
        zip.push(&code.reversed());
    }
    let encoded_zlib = zip.into_bytes();

    let encoded = Encoded {
        input_value: input.to_vec(),
        input_len: input.len(),
        input_bits: input.len() * 8,
        input_entropy: entropy_of_bytes(input),
        output_bits,
        output_rate: output_bits as f64 / (input.len() * 8) as f64 * 100.0,
        output_entropy: entropy_of_bytes(&encoded_zlib),
        prefix_table,
        encoded_str,
        encoded_zlib,
    };

    let result = serde_json::to_string(&encoded).unwrap();

    result
}

#[wasm_bindgen]
pub fn decode(input: &[u8], len: usize) -> Result<Vec<u8>, DecodeError> {
    let mut reader = BitStreamReader::new(input);

    let bfinal = reader
        .read(BitSize::Bit1)
        .ok_or(DecodeError::InvalidInput)?;
    let btype = reader
        .read(BitSize::Bit2)
        .ok_or(DecodeError::InvalidInput)?;
    let hlit = reader
        .read(BitSize::Bit5)
        .ok_or(DecodeError::InvalidInput)?;
    let hdist = reader
        .read(BitSize::Bit5)
        .ok_or(DecodeError::InvalidInput)?;

    if bfinal != 0b1 || btype != 0b10 || hlit != 0 || hdist != 0 {
        return Err(DecodeError::InvalidData);
    }

    let mut prefixes = Vec::new();
    CanonicalPrefixCoder::decode_prefix_tables(&mut reader, &mut prefixes, &[HLIT])
        .map_err(|_| DecodeError::InvalidData)?;
    let prefixes = prefixes
        .iter()
        .enumerate()
        .filter_map(|(index, &v)| (v > 0).then(|| (index, v)))
        .collect::<Vec<_>>();
    let decoder = CanonicalPrefixDecoder::new(&prefixes);

    let mut output = Vec::with_capacity(len);
    while output.len() < len {
        let code = decoder
            .decode(&mut reader)
            .map_err(|_| DecodeError::InvalidData)?;
        output.push(code as u8);
    }

    Ok(output)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Encoded {
    pub input_value: Vec<u8>,
    pub input_len: usize,
    pub input_bits: usize,
    pub input_entropy: f64,
    pub output_bits: usize,
    pub output_rate: f64,
    pub output_entropy: f64,
    pub prefix_table: Vec<PrefixTableEntry>,
    pub encoded_str: Vec<String>,
    pub encoded_zlib: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrefixTableEntry {
    pub symbol: usize,
    pub symbol_char: String,
    pub freq: usize,
    pub freq_rate: f64,
    pub len: u8,
    pub code: String,
}

#[wasm_bindgen]
pub enum DecodeError {
    InvalidInput,
    InvalidData,
}

fn _entropy_of<T>(data: &[(T, usize)]) -> f64 {
    let total_size = data.iter().map(|v| v.1).sum::<usize>() as f64;
    let mut entropy = 0.0;
    for (_, count) in data.iter() {
        let p = *count as f64 / total_size;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn entropy_of_bytes(input: &[u8]) -> f64 {
    let mut freq_table = vec![0usize; 256];
    input.iter().for_each(|&p| {
        freq_table[p as usize] += 1;
    });
    let freq_table = freq_table
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| (v > 0).then(|| (i as u8, v)))
        .collect::<Vec<_>>();
    _entropy_of(&freq_table)
}
