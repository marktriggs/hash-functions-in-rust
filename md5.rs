use std::fs::File;
use std::io::Read;

fn leftrotate(n: u32, pos: usize) -> u32 {
    assert!(pos <= 32);
    (n << pos) | (n >> (32 - pos))
}

const SHIFTS: [usize; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

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

fn md5(input: &[u8]) -> String {
    let mut a0 = 0x67452301u32;
    let mut b0 = 0xefcdab89u32;
    let mut c0 = 0x98badcfeu32;
    let mut d0 = 0x10325476u32;

    let preprocessed_message = preprocess(input);

    for chunk in preprocessed_message.chunks(64) {
        // Little endian here too
        let mut m: Vec<u32> = chunk
            .chunks(4)
            .map(|int32_bytes| {
                ((int32_bytes[3] as u32) << 24)
                    | ((int32_bytes[2] as u32) << 16)
                    | ((int32_bytes[1] as u32) << 8)
                    | ((int32_bytes[0] as u32) << 0)
            }).collect();

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let mut f;
            let mut g;

            if i <= 15 {
                f = (b & c) | ((!b) & d);
                g = i;
            } else if i <= 31 {
                f = (d & b) | ((!d) & c);
                g = (5 * i + 1) % 16;
            } else if i <= 47 {
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                f = c ^ (b | (!d));
                g = (7 * i) % 16;
            }

            f = f.wrapping_add(a).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(leftrotate(f, SHIFTS[i]));
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = String::new();

    for v in &[a0, b0, c0, d0] {
        result.push_str(&format!(
            "{:02x}{:02x}{:02x}{:02x}",
            (v >> 0) as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            (v >> 24) as u8));
    }

    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = File::open(args[1].clone()).expect("Failed to open input file");
    let content: Vec<u8> = file.bytes().map(Result::unwrap).collect();
    println!("{} {}", args[1], md5(&content));
}
