use std::{cmp::min, io};

use crate::{table::*, utils::*};

// According to `man` this is the default level.
const ZSTD_LEVEL: i32 = 3;

pub trait Serialize {
    fn serialize<W: io::Write>(&self, output: &mut W) -> io::Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize<R: io::Read>(input: &mut R) -> io::Result<Self>;
}

// ===== Deserialize =====

impl Deserialize for Table {
    fn deserialize<R: io::Read>(input: &mut R) -> io::Result<Self> {
        let n_rows = read_u64(input)?.try_into().unwrap();
        let n_columns = read_u64(input)?;
        let col_types = {
            let mut v = vec![0u8; n_columns.try_into().unwrap()];
            input.read_exact(v.as_mut_slice())?;
            v.into_iter()
                .map(|b| {
                    ColumnType::try_from(b).map_err(|b| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("bad column type {b}"))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let mut batches = Vec::<Batch>::new();
        let mut n_rows_remaining = n_rows;
        while n_rows_remaining > 0 {
            let n_batch_rows = min(n_rows_remaining, BATCH_SIZE);
            let batch = deserialize_batch(input, n_batch_rows, col_types.as_slice())?;
            batches.push(batch);
            n_rows_remaining -= n_batch_rows;
        }

        Ok(Table::new(batches).unwrap())
    }
}

fn deserialize_batch<R: io::Read>(
    input: &mut R,
    n_rows: usize,
    col_types: &[ColumnType],
) -> io::Result<Batch> {
    let columns = col_types
        .iter()
        .map(|&col_type| deserialize_column_chunk(input, n_rows, col_type))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Batch {
        len: n_rows,
        columns: columns,
    })
}

fn deserialize_column_chunk<R: io::Read>(
    input: &mut R,
    n_rows: usize,
    col_type: ColumnType,
) -> io::Result<ColumnChunk> {
    let data_len = read_u64(input)?.try_into().unwrap();
    let mut data = vec![0u8; data_len];
    input.read_exact(&mut data)?;

    match col_type {
        ColumnType::Int => deserialize_int_column_chunk(&data, n_rows),
        ColumnType::Bool => deserialize_bool_column_chunk(&data, n_rows),
        ColumnType::String => deserialize_string_column_chunk(&data, n_rows),
    }
}

fn deserialize_int_column_chunk(data: &[u8], n_rows: usize) -> io::Result<ColumnChunk> {
    let mut input = data;
    let mut result = Vec::with_capacity(n_rows);
    let mut last = 0i64;
    for _ in 0..n_rows {
        let delta = read_vle_i64(&mut input)?;
        let current = last.wrapping_add(delta);
        result.push(current);
        last = current;
    }

    if !input.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "leftover data in int column chunk",
        ))
    } else {
        Ok(ColumnChunk::Int(result))
    }
}

fn deserialize_bool_column_chunk(data: &[u8], n_rows: usize) -> io::Result<ColumnChunk> {
    let mut result = Vec::with_capacity(n_rows);

    let mut remaining = n_rows;
    for byte in data {
        let c = min(remaining, 8);
        result.extend((0..8).map(|i| (byte >> i) & 1 == 1).take(c));
        remaining -= c;
    }

    if remaining > 0 {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "leftover data in bool column chunk",
        ))
    } else {
        Ok(ColumnChunk::Bool(result))
    }
}

fn deserialize_string_column_chunk(data: &[u8], n_rows: usize) -> io::Result<ColumnChunk> {
    let mut input = data;

    let ends = deserialize_string_column_chunk_ends(&mut input, n_rows)?;
    let data = {
        let mut v = Vec::new();
        zstd::stream::copy_decode(&mut input, &mut v)?;
        v.try_into().unwrap()
    };

    Ok(ColumnChunk::String {
        data: data,
        ends: ends,
    })
}

fn deserialize_string_column_chunk_ends<R: io::Read>(
    input: &mut R,
    n_rows: usize,
) -> io::Result<Vec<usize>> {
    let mut result = Vec::with_capacity(n_rows);
    let mut len = 0usize;
    let mut end = 0usize;

    for _ in 0..n_rows {
        let delta = read_vle_i64(input)?;
        len = len.checked_add_signed(delta.try_into().unwrap()).unwrap();
        end += len;
        result.push(end);
    }

    Ok(result)
}

// ===== Serialize =====

impl Serialize for Table {
    fn serialize<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        write_u64(output, self.len() as u64)?;

        let col_types = self
            .batches()
            .get(0)
            .map(|b| b.columns.iter().map(|c| c.col_type()).collect())
            .unwrap_or(Vec::new());

        write_u64(output, col_types.len() as u64)?;
        let col_bytes = col_types.iter().map(|&t| t as u8).collect::<Vec<_>>();
        output.write_all(&col_bytes)?;

        for batch in self.batches() {
            batch.serialize(output)?;
        }

        Ok(())
    }
}

impl Serialize for Batch {
    fn serialize<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        for col in self.columns.iter() {
            col.serialize(output)?;
        }

        Ok(())
    }
}

impl Serialize for ColumnChunk {
    fn serialize<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let mut buffer = Vec::new();
        match self {
            ColumnChunk::Int(data) => serialize_int_column_chunk(&mut buffer, data)?,
            ColumnChunk::Bool(data) => serialize_bool_column_chunk(&mut buffer, data)?,
            ColumnChunk::String { data, ends } => {
                serialize_string_column_chunk(&mut buffer, data, ends)?
            }
        };
        write_u64(output, buffer.len() as u64)?;
        output.write_all(&buffer)
    }
}

fn serialize_int_column_chunk<W: io::Write>(output: &mut W, data: &[i64]) -> io::Result<()> {
    let mut last = 0i64;
    for &n in data {
        write_vle_i64(output, n.wrapping_sub(last))?;
        last = n;
    }
    Ok(())
}

fn serialize_bool_column_chunk<W: io::Write>(output: &mut W, data: &[bool]) -> io::Result<()> {
    let bytes = data
        .chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u8, |acc, (i, &b)| acc | ((b as u8) << i))
        })
        .collect::<Vec<_>>();
    output.write_all(&bytes)
}

fn serialize_string_column_chunk<W: io::Write>(
    output: &mut W,
    data: &str,
    ends: &[usize],
) -> io::Result<()> {
    let mut prev_end = 0usize;
    let mut prev_len = 0i64;
    for &end in ends {
        let len = (end - prev_end) as i64;
        write_vle_i64(output, len - prev_len)?;
        prev_end = end;
        prev_len = len;
    }

    zstd::stream::copy_encode(data.as_bytes(), &mut *output, ZSTD_LEVEL)
}
