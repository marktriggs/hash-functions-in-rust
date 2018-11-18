use std::fs::File;
use std::io::Read;

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

fn leftrotate(n: u32, pos: usize) -> u32 {
    assert!(pos <= 32);
    (n << pos) | (n >> (32 - pos))
}

fn sha1(input: &[u8]) -> String {
    let mut h0 = 0x67452301u32;
    let mut h1 = 0xEFCDAB89u32;
    let mut h2 = 0x98BADCFEu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xC3D2E1F0u32;

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

        for i in 16..80 {
            w.push(0);

            w[i] = leftrotate(w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16], 1)
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let mut f;
            let mut k;

            if i <= 19 {
                f = (b & c) | ((!b) & d);
                k = 0x5A827999;
            } else if i <= 39 {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if i <= 59 {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8F1BBCDC;
            } else {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }

            let mut temp = leftrotate(a, 5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);

            e = d;
            d = c;
            c = leftrotate(b, 30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
}


fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], sha1(&content));
}
