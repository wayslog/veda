const C1: u32 = 0x85eb_ca6b;
const C2: u32 = 0xc2b2_ae35;
const R1: u32 = 16;
const R2: u32 = 13;
const M: u32 = 5;
const N: u32 = 0xe654_6b64;

pub fn murmur3_32(source: &[u8], seed: u32) -> u32 {
    let buf = source.as_ref();

    let mut processed = 0;
    let mut state = seed;

    let mut iter = buf.array_chunks::<4>();

    while let Some(buffer) = iter.next() {
        processed += 4;
        let k = u32::from_le_bytes(*buffer);
        state ^= calc_k(k);
        state = state.rotate_left(R2);
        state = (state.wrapping_mul(M)).wrapping_add(N);
    }

    let buffer = iter.remainder();
    match buffer.len() {
        3 => {
            processed += 3;
            let k: u32 =
                ((buffer[2] as u32) << 16) | ((buffer[1] as u32) << 8) | (buffer[0] as u32);
            state ^= calc_k(k);
        }
        2 => {
            processed += 2;
            let k: u32 = ((buffer[1] as u32) << 8) | (buffer[0] as u32);
            state ^= calc_k(k);
        }
        1 => {
            processed += 1;
            let k: u32 = buffer[0] as u32;
            state ^= calc_k(k);
        }
        0 => {}
        _ => panic!("Internal buffer state failure"),
    }
    finish(state, processed)
}

fn finish(state: u32, processed: u32) -> u32 {
    let mut hash = state;
    hash ^= processed as u32;
    hash ^= hash.wrapping_shr(R1);
    hash = hash.wrapping_mul(C1);
    hash ^= hash.wrapping_shr(R2);
    hash = hash.wrapping_mul(C2);
    hash ^= hash.wrapping_shr(R1);
    hash
}

fn calc_k(k: u32) -> u32 {
    const C1: u32 = 0xcc9e_2d51;
    const C2: u32 = 0x1b87_3593;
    const R1: u32 = 15;
    k.wrapping_mul(C1).rotate_left(R1).wrapping_mul(C2)
}

#[test]
fn test_murmur3() {
    let x = murmur3_32("hello world".as_bytes(), 0);
    assert_eq!(x, 1586663183);
}
