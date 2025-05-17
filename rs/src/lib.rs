//! Prefix Code Tester

extern crate alloc;

use compress::{
    deflate::Deflate,
    entropy::prefix::{CanonicalPrefixCoder, HuffmanTreeNode, PermutationFlavor},
    num::bits::{BitSize, BitStreamWriter, VarBitValue},
};
use core::fmt::Display;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode_chc(input: &[u8], max_len: u8) -> Result<String, String> {
    _encode_chc(input, max_len).map_err(|e| format!("{}", e))
}

pub fn _encode_chc(input: &[u8], max_len: u8) -> Result<String, EncodeError> {
    let max_len = BitSize::new(max_len).ok_or(EncodeError::InvalidInput)?;

    let mut freq_table = Vec::new();
    freq_table.resize(256, 0usize);
    for &byte in input.iter() {
        freq_table[byte as usize] += 1;
    }
    let freq_table = freq_table
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| (v > 0).then(|| (i as u8, v)))
        .collect::<Vec<_>>();

    let mut prefix_table = Vec::new();
    let mut result_tree = Vec::new();
    prefix_table.resize(258, None);
    for item in
        CanonicalPrefixCoder::generate_prefix_table(&freq_table, max_len, Some(&mut result_tree))
    {
        prefix_table[item.0 as usize] = Some(item.1);
    }

    let mut encoded_str = Vec::new();
    let mut encoded_codes = Vec::new();
    let mut output_bits = 0;
    for &byte in input.iter() {
        let code = prefix_table[byte as usize].ok_or(EncodeError::InvalidInput)?;
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
    let result_prefix_table = freq_table
        .iter()
        .zip(prefix_table2.iter())
        .map(|(freq, prefix)| {
            let symbol_char = stringify_char(freq.0 as u8);
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
    {
        let zlib_meta = CanonicalPrefixCoder::encode_single_prefix_table(
            &prefix_table,
            PermutationFlavor::Deflate,
        )
        .unwrap();

        zip.push_bool(true); // BFINAL: true
        zip.push(VarBitValue::new(BitSize::Bit2, 0b10)); // BTYPE: 10 dynamic huffman
        zip.push(VarBitValue::new(BitSize::Bit5, 0)); // HLIT: 0
        zip.push(VarBitValue::new(BitSize::Bit5, 0)); // HDIST: 0
        zip.push_nibble(zlib_meta.hclen); // HCLEN
        zip.push_slice(&zlib_meta.prefix_table);
        zip.push_slice(&zlib_meta.content);
        for code in encoded_codes.iter() {
            zip.push(code.reversed());
        }
    }
    let encoded_zlib = zip.into_bytes();

    let mut webp = BitStreamWriter::new();
    match prefix_table2.len() {
        1 => {
            webp.push_bool(true); // simple code
            webp.push_bool(false); // num_symbols = 1
            let symbol = prefix_table2[0].0 as u8;
            if symbol < 2 {
                webp.push_bool(false); // is_first_8bit = false
                webp.push_bool(symbol != 0);
            } else {
                webp.push_bool(true); // is_first_8bit = true
                webp.push_byte(symbol);
            }
        }
        2 => {
            webp.push_bool(true); // simple code
            webp.push_bool(true); // num_symbols = 2
            let first = prefix_table2[0].0 as u8;
            if first < 2 {
                webp.push_bool(false); // is_first_8bit = false
                webp.push_bool(first != 0);
            } else {
                webp.push_bool(true); // is_first_8bit = true
                webp.push_byte(first);
            }
            webp.push_byte(prefix_table2[1].0 as u8);

            for code in encoded_codes.iter() {
                webp.push(code.reversed());
            }
        }
        _ => {
            let webp_meta = CanonicalPrefixCoder::encode_single_prefix_table(
                &prefix_table,
                PermutationFlavor::WebP,
            )
            .unwrap();

            webp.push_bool(false); // normal code
            webp.push_nibble(webp_meta.hclen);
            webp.push_slice(&webp_meta.prefix_table);
            webp.push_bool(false); // max_symbol = default
            webp.push_slice(&webp_meta.content);

            for code in encoded_codes.iter() {
                webp.push(code.reversed());
            }
        }
    }
    let encoded_webp = webp.into_bytes();

    let huffman_tree = if let Some(tree) = result_tree.get(0) {
        let mut huffman_tree: Vec<String> = Vec::new();
        render_huffman_tree(&mut huffman_tree, tree, 0);
        huffman_tree.join("\n")
    } else {
        "".to_owned()
    };

    let encoded = EncodeChcResult {
        input_len: input.len(),
        input_bits: input.len() * 8,
        input_entropy: entropy_of_bytes(input),
        output_bits,
        output_entropy: entropy_of_bytes(&encoded_zlib),
        webp_entropy: entropy_of_bytes(&encoded_webp),
        prefix_table: result_prefix_table,
        encoded_str,
        encoded_zlib,
        encoded_webp,
        huffman_tree,
    };

    let result = serde_json::to_string(&encoded).unwrap();
    Ok(result)
}

#[wasm_bindgen]
pub fn decode_chc(input: &[u8], len: usize) -> Result<Vec<u8>, DecodeError> {
    Deflate::inflate(input, len).map_err(|_| DecodeError::InvalidData)
}

fn stringify_char(data: u8) -> String {
    if data < 0x20 || data > 0x7e {
        format!("\"\\x{:02x}\"", data)
    } else {
        format!("\"{}\"", data as char)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncodeChcResult {
    pub input_len: usize,
    pub input_bits: usize,
    pub input_entropy: f64,
    pub output_bits: usize,
    pub output_entropy: f64,
    pub prefix_table: Vec<PrefixTableEntry>,
    pub encoded_str: Vec<String>,
    pub encoded_zlib: Vec<u8>,
    pub encoded_webp: Vec<u8>,
    pub webp_entropy: f64,
    pub huffman_tree: String,
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

fn render_huffman_tree(output: &mut Vec<String>, item: &HuffmanTreeNode<u8>, nest: usize) {
    let current_indent = " ".repeat(nest * 2);
    if let Some(&symbol) = item.symbol() {
        output.push(format!(
            "{current_indent}{}: {}",
            item.freq(),
            stringify_char(symbol),
        ));
    } else {
        output.push(format!("{current_indent}{}:", item.freq()));
        if let Some(left) = item.left() {
            render_huffman_tree(output, left, nest + 1);
        }
        if let Some(right) = item.right() {
            render_huffman_tree(output, right, nest + 1);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncodeError {
    InvalidInput,
}

impl Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EncodeError::InvalidInput => {
                write!(f, "Invalid input (e.g. not enough Bit Length)")
            }
        }
    }
}
