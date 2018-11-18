use std::fs::File;
use std::io::Read;

fn rightrotate(n: u32, pos: usize) -> u32 {
    assert!(pos <= 32);
    (n >> pos) | (n << (32 - pos))
}

fn preprocess(message: &[u8]) -> Vec<u8> {
    let message_length: u64 = message.len() as u64 * 8;
    let mut result = message.to_owned();

    result.push(0x80);

    while ((result.len() * 8) + 64) % 512 != 0 {
        result.push(0);
    }

    // u64 -> big endian bytes
    for b in 1..=8 {
        result.push((message_length >> (64 - (b * 8))) as u8);
    }

    result
}

fn sha256(input: &[u8]) -> String {
    let mut h0 = 0x6a09e667u32;
    let mut h1 = 0xbb67ae85u32;
    let mut h2 = 0x3c6ef372u32;
    let mut h3 = 0xa54ff53au32;
    let mut h4 = 0x510e527fu32;
    let mut h5 = 0x9b05688cu32;
    let mut h6 = 0x1f83d9abu32;
    let mut h7 = 0x5be0cd19u32;

    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let preprocessed_message = preprocess(input);

    for chunk in preprocessed_message.chunks(64) {
        let mut w: Vec<u32> = chunk
            .chunks(4)
            .map(|int32_bytes| {
                ((int32_bytes[0] as u32) << 24)
                    | ((int32_bytes[1] as u32) << 16)
                    | ((int32_bytes[2] as u32) << 8)
                    | ((int32_bytes[3] as u32) << 0)
            }).collect();

        w.resize(64, 0);

        for i in 16..64 {
            let s0 = rightrotate(w[i - 15], 7) ^ rightrotate(w[i - 15], 18) ^ (w[i - 15] >> 3);
            let s1 = rightrotate(w[i - 2], 17) ^ rightrotate(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for i in 0..64 {
            let s1 = rightrotate(e, 6) ^ rightrotate(e, 11) ^ rightrotate(e, 25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = rightrotate(a, 2) ^ rightrotate(a, 13) ^ rightrotate(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h0, h1, h2, h3, h4, h5, h6, h7
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input <file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], sha256(&content));
}
