use std::fs::File;
use std::io::Read;


const IV: &[u64] = &[
    0x6A09E667F3BCC908,
    0xBB67AE8584CAA73B,
    0x3C6EF372FE94F82B,
    0xA54FF53A5F1D36F1,
    0x510E527FADE682D1,
    0x9B05688C2B3E6C1F,
    0x1F83D9ABFB41BD6B,
    0x5BE0CD19137E2179,
];

const SIGMA: &[&[usize]] = &[
    &[0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15],
    &[14, 10, 4,  8,  9,  15, 13, 6,  1,  12, 0,  2,  11, 7,  5,  3 ],
    &[11, 8,  12, 0,  5,  2,  15, 13, 10, 14, 3,  6,  7,  1,  9,  4 ],
    &[7,  9,  3,  1,  13, 12, 11, 14, 2,  6,  5,  10, 4,  0,  15, 8 ],
    &[9,  0,  5,  7,  2,  4,  10, 15, 14, 1,  11, 12, 6,  8,  3,  13],
    &[2,  12, 6,  10, 0,  11, 8,  3,  4,  13, 7,  5,  15, 14, 1,  9 ],
    &[12, 5,  1,  15, 14, 13, 4,  10, 0,  7,  6,  3,  9,  2,  8,  11],
    &[13, 11, 7,  14, 12, 1,  3,  9,  5,  0,  15, 4,  8,  6,  2,  10],
    &[6,  15, 14, 9,  11, 3,  0,  8,  12, 2,  13, 7,  1,  4,  10, 5 ],
    &[10, 2,  8,  4,  7,  6,  1,  5,  15, 11, 9,  14, 3,  12, 13, 0 ],
];

fn pad(buffer: &mut Vec<u8>, size: usize) {
    let m = buffer.len() % size;

    let padding = size - m;
    buffer.extend(vec![0; padding].iter());
}

fn rightrotate(n: u64, pos: usize) -> u64 {
    assert!(pos <= 64);
    (n >> pos) | (n << (64 - pos))
}

fn mix(v: &mut Vec<u64>,
       a: usize, b: usize, c: usize, d: usize,
       x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = rightrotate((v[d] ^ v[a]) as u64, 32);

    v[c] = v[c].wrapping_add(v[d]);
    v[b] = rightrotate((v[b] ^ v[c]) as u64, 24);

    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = rightrotate((v[d] ^ v[a]) as u64, 16);

    v[c] = v[c].wrapping_add(v[d]);
    v[b] = rightrotate((v[b] ^ v[c]) as u64, 63);
}

fn compress(h: &mut Vec<u64>, chunk: &Vec<u8>, t: u128, is_last_block: bool) {
    let mut v = h.clone();
    v.extend_from_slice(IV);

    v[12] ^= ((t << 64) >> 64) as u64;
    v[13] ^= (t >> 64) as u64;

    if is_last_block {
        v[14] = !v[14];
    }

    let m: Vec<u64> = chunk.chunks(8).map(|eight_bytes| {
        // little-endian u64
        let mut result: u64 = 0;
        for b in 0..8 {
            result = (result << 8) | eight_bytes[7 - b] as u64
        }

        result
    }).collect();

    for i in 0..12 {
        let s: &[usize] = SIGMA[i % 10];

        mix(&mut v, 0, 4, 8,  12, m[s[0]], m[s[1]]);
        mix(&mut v, 1, 5, 9,  13, m[s[2]], m[s[3]]);
        mix(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        mix(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

        mix(&mut v, 0, 5, 10, 15, m[s[8]],  m[s[9]]);
        mix(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        mix(&mut v, 2, 7, 8,  13, m[s[12]], m[s[13]]);
        mix(&mut v, 3, 4, 9,  14, m[s[14]], m[s[15]]);
    }

    for i in 0..8 {
        h[i] ^= v[i];
    }

    for i in 0..8 {
        h[i] ^= v[i + 8];
    }
}

fn blake2(input: &[u8], key: Option<&[u8]>, hashlen: usize) -> String {
    if hashlen < 1 || hashlen > 64 {
        panic!("Requested hash must be between 1 and 64 bytes");
    }

    let key_length = if key.is_some() {
        let len = key.unwrap().len();
        if len > 64 {
            panic!("Key too large");
        } else {
            len
        }
    } else {
        0
    };

    let mut h = IV.to_vec();

    h[0] ^= 0x01010000u64 | (key_length << 8) as u64 | hashlen as u64;

    let mut bytes_compressed: u128 = 0;
    let mut bytes_remaining: u128 = input.len() as u128;

    // Apply key if we have one
    let mut m: Vec<u8> = if key_length > 0 {
        let mut key = key.unwrap().to_vec();
        pad(&mut key, 128);

        bytes_remaining += 128;

        let mut result = Vec::with_capacity(key.len() + input.len());
        result.append(&mut key);
        result.extend_from_slice(input);
        result
    } else {
        input.to_vec()
    };


    while bytes_remaining > 128 {
        let chunk: Vec<u8> = m.drain(0..128).collect();
        bytes_compressed += 128;
        bytes_remaining -= 128;

        compress(&mut h, &chunk, bytes_compressed, false);
    }


    bytes_compressed += bytes_remaining;
    pad(&mut m, 128);

    compress(&mut h, &m, bytes_compressed, true);

    // Result ← first cbHashLen bytes of little endian state vector h
    let mut result = String::new();

    let mut generated_bytes: Vec<u8> = Vec::with_capacity(h.len() * 8);

    for n in h {
        // u64 -> Little endian bytes
        let mut acc = n;
        for _ in 0..8 {
            generated_bytes.push((acc & 0xFFu64) as u8);
            acc = acc >> 8;
        }
    }

    for b in &generated_bytes[0..hashlen] {
        result.push_str(&format!("{:02x}", b));
    }

    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input <file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], blake2(&content, None, 64));
}
