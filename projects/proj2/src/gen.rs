use std::fs::File;
use std::io::{self, BufWriter, Write};

use proj2::deser::Serialize;
use proj2::table::*;

const N_ROWS: usize = 1500;

pub fn main() -> io::Result<()> {
    let table = make_table();

    let path = std::env::args().nth(1).expect("usage: gen <output file>");
    let mut output = BufWriter::new(File::create(&path)?);
    table.serialize(&mut output)?;
    output.flush()?;

    println!("Wrote table with {} rows to {path}.", table.len());
    Ok(())
}

fn make_table() -> Table {
    let batches = vec![make_batch(BATCH_SIZE), make_batch(N_ROWS - BATCH_SIZE)];
    Table::new(batches).unwrap()
}

fn make_batch(len: usize) -> Batch {
    let rows = 0..len;
    let columns = vec![
        ColumnChunk::Int(rows.clone().map(|i| if i % 2 == 0 { 41 } else { 43 }).collect()),
        ColumnChunk::Bool(rows.clone().map(|i| i % 2 == 1).collect()),
        string_chunk(rows.clone().map(|i| if i % 2 == 0 { "lorem" } else { "ipsum" })),
        string_chunk(rows.clone().map(|i| if i % 2 == 0 { "foo" } else { "bar" })),
    ];

    Batch {
        len: rows.len(),
        columns,
    }
}

fn string_chunk<'a>(strings: impl Iterator<Item = &'a str>) -> ColumnChunk {
    let mut data = String::new();
    let mut ends = Vec::new();
    for s in strings {
        data.push_str(s);
        ends.push(data.len());
    }
    ColumnChunk::String { data, ends }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proj2::deser::Deserialize;

    #[test]
    fn round_trip() {
        let table = make_table();

        let mut pretend_out = Vec::new();
        table.serialize(&mut pretend_out).unwrap();

        let mut pretend_in = pretend_out.as_slice();
        let read_back = Table::deserialize(&mut pretend_in).unwrap();

        assert_eq!(table, read_back);
        assert!(pretend_in.is_empty());
    }
}
