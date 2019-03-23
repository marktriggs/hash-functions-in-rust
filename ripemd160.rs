use std::fs::File;
use std::io::Read;

fn preprocess(message: &[u8]) -> Vec<u8> {
    let message_length: u64 = message.len() as u64 * 8;
    let mut result = message.to_owned();

    result.push(0x80);

    while ((result.len() * 8) + 64) % 512 != 0 {
        result.push(0);
    }

    // u64 -> little endian bytes
    for b in 0..8 {
        result.push((message_length >> (b * 8)) as u8);
    }

    result
}

// Non-linear functions at bit-level
fn func(j: usize, x: u32, y: u32, z: u32) -> u32 {
    match j {
        0...15 => x ^ y ^ z,
        16...31 => (x & y) | (!x & z),
        32...47 => (x | !y) ^ z,
        48...63 => (x & z) | (y & !z),
        64...79 => x ^ (y | !z),
        _ => {
            unreachable!();
        }
    }
}

const R_OFFSETS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5,
    2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8, 12, 4,
    13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
];

const R_P_OFFSETS: &[usize] = &[
    5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12,
    4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11, 15, 0, 5,
    12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
];

fn word_select(i: usize, j: usize, msg: &[u8], offsets: &[usize]) -> u32 {
    let word_offset = (i * 16 * 4) + (offsets[j] * 4);

    // little-endian here
    u32::from_be_bytes([
        msg[word_offset + 3],
        msg[word_offset + 2],
        msg[word_offset + 1],
        msg[word_offset + 0],
    ])
}

fn constant_k(j: usize) -> u32 {
    match j {
        0...15 => 0x00000000u32,
        16...31 => 0x5A827999u32,
        32...47 => 0x6ED9EBA1u32,
        48...63 => 0x8F1BBCDCu32,
        64...79 => 0xA953FD4Eu32,
        _ => {
            unreachable!();
        }
    }
}

fn constant_k_p(j: usize) -> u32 {
    match j {
        0...15 => 0x50A28BE6u32,
        16...31 => 0x5C4DD124u32,
        32...47 => 0x6D703EF3u32,
        48...63 => 0x7A6D76E9u32,
        64...79 => 0x00000000u32,
        _ => {
            unreachable!();
        }
    }
}

const ROTATIONS: &[u32] = &[
    11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15,
    9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14, 15, 14,
    15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6,
];

const ROTATIONS_P: &[u32] = &[
    8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12,
    7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11, 14, 14,
    6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11,
];

// Based on pseudocode from Appendix A:
// https://homes.esat.kuleuven.be/~bosselae/ripemd160/pdf/AB-9601/AB-9601.pdf
//
fn ripemd160(input: &[u8]) -> String {
    let preprocessed_message = preprocess(input);

    // Number of 16-word blocks in our padded message, where word-size is
    // 32-bits.  Called `t` in the original paper, but want to keep that
    // variable free for the `T` variable used in each round.
    assert!(preprocessed_message.len() % (4 * 16) == 0);
    let rounds = preprocessed_message.len() / 4 / 16;

    let mut h0 = 0x67452301u32;
    let mut h1 = 0xEFCDAB89u32;
    let mut h2 = 0x98BADCFEu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xC3D2E1F0u32;

    // a corresponds to A in the original paper; a_p corresponds to A'
    for i in 0..rounds {
        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut a_p = h0;
        let mut b_p = h1;
        let mut c_p = h2;
        let mut d_p = h3;
        let mut e_p = h4;

        let mut t;

        for j in 0..80 {
            t = a
                .wrapping_add(func(j, b, c, d))
                .wrapping_add(word_select(i, j, &preprocessed_message, R_OFFSETS))
                .wrapping_add(constant_k(j))
                .rotate_left(ROTATIONS[j])
                .wrapping_add(e);
            a = e;
            e = d;
            d = c.rotate_left(10);
            c = b;
            b = t;

            t = a_p
                .wrapping_add(func(79 - j, b_p, c_p, d_p))
                .wrapping_add(word_select(i, j, &preprocessed_message, R_P_OFFSETS))
                .wrapping_add(constant_k_p(j))
                .rotate_left(ROTATIONS_P[j])
                .wrapping_add(e_p);

            a_p = e_p;
            e_p = d_p;
            d_p = c_p.rotate_left(10);
            c_p = b_p;
            b_p = t;
        }

        t = h1.wrapping_add(c).wrapping_add(d_p);
        h1 = h2.wrapping_add(d).wrapping_add(e_p);
        h2 = h3.wrapping_add(e).wrapping_add(a_p);
        h3 = h4.wrapping_add(a).wrapping_add(b_p);
        h4 = h0.wrapping_add(b).wrapping_add(c_p);
        h0 = t;
    }

    let mut result = String::new();

    for v in &[h0, h1, h2, h3, h4] {
        result.push_str(&format!(
            "{:02x}{:02x}{:02x}{:02x}",
            (v >> 0) as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8
        ));
    }

    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], ripemd160(&content));
}
