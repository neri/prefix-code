extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

mod prefix;
pub use prefix::*;

pub mod bits;
pub mod stats;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncodeError {
    InvalidInput,
    InvalidData,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidInput,
    InvalidData,
    OutOfMemory,
}

#[test]
fn canonical_prefix_code() {
    let input = "abracadabra";
    // let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    let mut freq_table = Vec::new();
    freq_table.resize(256, 0usize);
    for byte in input.bytes() {
        freq_table[byte as usize] += 1;
    }
    let freq_table = freq_table
        .iter()
        .enumerate()
        .filter(|(_, v)| **v > 0)
        .map(|(i, &v)| (i as u8, v))
        .collect::<Vec<_>>();

    let mut table = Vec::new();
    table.resize(256, None);
    for item in
        prefix::CanonicalPrefixCoder::generate_prefix_table(&freq_table, bits::BitSize16::Bit16)
    {
        table[item.0 as usize] = bits::AnyBitValue::new(item.1, item.2);
    }

    let meta = prefix::CanonicalPrefixCoder::encode_single_prefix_table(table.as_ref()).unwrap();

    println!("INPUT:\n {:?}", input);

    println!("FREQUENCY OF SYMBOLS:");
    for (index, value) in freq_table.iter() {
        println!(" {:02x} {:?} {}", index, *index as u8 as char, *value);
    }

    println!("PREFIX TABLE:");
    for (index, item) in table.iter().enumerate() {
        if let Some(item) = item {
            println!(
                " {:02x} {:?} {} {}",
                index,
                index as u8 as char,
                item.size(),
                item
            );
        }
    }

    println!("ENCODED:");
    let mut acc = 0;
    for byte in input.bytes() {
        let code = table[byte as usize].unwrap();
        print!(" {}", code);
        acc += code.size() as usize;
    }
    println!("");
    let input_len = input.len() * 8;
    println!(
        "# {} bits <= {} bits {:.03}%",
        acc,
        input_len,
        acc as f64 / input_len as f64 * 100.0
    );

    println!("META PREFIX TABLE:");
    let mut acc = 0;
    print!(" HCLEN {},", meta.hclen);
    for code in meta.prefix_table.iter() {
        print!(" {}", code.value());
        acc += code.size() as usize;
    }
    println!("");
    for code in meta.payload.iter() {
        print!(" {}", code);
        acc += code.size() as usize;
    }
    println!("");
    println!("# {} bits", acc);

    todo!();
}
