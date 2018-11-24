use std::fs::File;
use std::io::Read;

const K: &[u64] = &[
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc, 0x3956c25bf348b538, 0x59f111f1b605d019,
    0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242,
    0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235,
    0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275,
    0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f,
    0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725,
    0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc,
    0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6,
    0x92722c851482353b, 0xa2bfe8a14cf10364, 0xa81a664bbc423001,
    0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218,
    0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc,
    0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915,
    0xc67178f2e372532b, 0xca273eceea26619c, 0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba,
    0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
 ];


fn rightrotate(n: u64, pos: usize) -> u64 {
    assert!(pos <= 64);
    (n >> pos) | (n << (64 - pos))
}

fn preprocess(message: &[u8]) -> Vec<u8> {
    let message_length: u128 = message.len() as u128 * 8;
    let mut result = message.to_owned();

    result.push(0x80);

    while ((result.len() * 8) + 128) % 1024 != 0 {
        result.push(0);
    }

    for b in 1..=16 {
        result.push((message_length >> (128 - (b * 8))) as u8);
    }

    result
}

fn sha512(input: &[u8]) -> String {
    let mut h0 = 0x6a09e667f3bcc908u64;
    let mut h1 = 0xbb67ae8584caa73bu64;
    let mut h2 = 0x3c6ef372fe94f82bu64;
    let mut h3 = 0xa54ff53a5f1d36f1u64;
    let mut h4 = 0x510e527fade682d1u64;
    let mut h5 = 0x9b05688c2b3e6c1fu64;
    let mut h6 = 0x1f83d9abfb41bd6bu64;
    let mut h7 = 0x5be0cd19137e2179u64;

    let preprocessed_message = preprocess(input);

    for chunk in preprocessed_message.chunks(128) {
        let mut w: Vec<u64> = chunk
            .chunks(8)
            .map(|int64_bytes| {
                ((int64_bytes[0] as u64) << 56)
                    | ((int64_bytes[1] as u64) << 48)
                    | ((int64_bytes[2] as u64) << 40)
                    | ((int64_bytes[3] as u64) << 32)
                    | ((int64_bytes[4] as u64) << 24)
                    | ((int64_bytes[5] as u64) << 16)
                    | ((int64_bytes[6] as u64) << 8)
                    | ((int64_bytes[7] as u64) << 0)
            }).collect();

        w.resize(80, 0);

        for i in 16..80 {
            let s0 = rightrotate(w[i - 15], 1) ^ rightrotate(w[i - 15], 8) ^ (w[i - 15] >> 7);
            let s1 = rightrotate(w[i - 2], 19) ^ rightrotate(w[i - 2], 61) ^ (w[i - 2] >> 6);
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

        for i in 0..80 {
            let s1 = rightrotate(e, 14) ^ rightrotate(e, 18) ^ rightrotate(e, 41);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = rightrotate(a, 28) ^ rightrotate(a, 34) ^ rightrotate(a, 39);
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
        "{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}{:016x}",
        h0, h1, h2, h3, h4, h5, h6, h7
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], sha512(&content));
}
