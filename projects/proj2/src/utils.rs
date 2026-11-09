use std::io;

pub fn zigzag(n: i64) -> u64 {
    (n >> 63) as u64 ^ (n << 1) as u64
}

pub fn unzigzag(n: u64) -> i64 {
    (n >> 1) as i64 ^ -((n & 1) as i64)
}

pub fn read_u64<R: io::Read>(input: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    input.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn write_u64<W: io::Write>(output: &mut W, n : u64) -> io::Result<()> {
    output.write_all(&n.to_le_bytes())
}

pub fn read_vle_i64<R: io::Read>(input: &mut R) -> io::Result<i64> {
    let mut zigzagged = 0u64;

    let mut shift = 0;
    loop {
        let byte = read_byte(input)?;
        zigzagged |= ((byte & 0b0111_1111) as u64) << shift;
        shift += 7;

        if byte & (1 << 7) == 0 {
            break;
        }
    }

    Ok(unzigzag(zigzagged))
}

pub fn write_vle_i64<W: io::Write>(output: &mut W, n: i64) -> io::Result<()> {
    let mut zigzagged = zigzag(n);

    loop {
        let mut byte = (zigzagged & 0b0111_1111) as u8;
        zigzagged >>= 7;

        if zigzagged != 0 {
            byte |= 0b1000_0000;
        }
        write_byte(output, byte)?;

        if zigzagged == 0 {
            break;
        }
    }

    Ok(())
}

fn read_byte<R: io::Read>(input: &mut R) -> io::Result<u8> {
    let mut buf = [0u8];
    input.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_byte<W: io::Write>(output: &mut W, byte: u8) -> io::Result<()> {
    output.write_all(&[byte])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zz_end_to_end() {
        let samples: [i64; _] = [0, 1, -1, 24, -42, 2 << 60, -(2 << 60)];
        for n in samples {
            assert_eq!(n, unzigzag(zigzag(n)));
        }
    }

    #[test]
    fn zz_stay_small() {
        let samples: [i64; _] = [1, 10, 100, 1000, 1_000_000];
        for n in samples {
            assert!(zigzag(n) <= 2 * n as u64);
            assert!(zigzag(-n) <= 2 * n as u64);
        }
    }

    #[test]
    fn vle_end_to_end() {
        let samples: [i64; _] = [0, 1, -1, 24, -42, 2 << 60, -(2 << 60)];

        let mut pretend_out = Vec::new();

        for n in samples {
            write_vle_i64(&mut pretend_out, n).unwrap();
        }

        let mut pretend_in = pretend_out.as_slice();

        let mut result = Vec::with_capacity(samples.len());
        for _ in samples {
            result.push(read_vle_i64(&mut pretend_in).unwrap());
        }

        assert_eq!(samples, *result);
        // no leftovers
        assert!(pretend_in.is_empty());
    }

    #[test]
    fn vle_short() {
        let samples: [i64; _] = [0, 1, -1, 24, -42];

        for n in samples {
            let mut pretend_out = Vec::new();
            write_vle_i64(&mut pretend_out, n).unwrap();
            assert!(pretend_out.len() == 1);
        }
    }
}
