use std::{
    env,
    fs::File,
    io::{self, BufReader},
};

use proj2::{deser::Deserialize, table::*};

fn main() -> io::Result<()> {
    let path = env::args().nth(1).expect("usage: proj2 <input file>");
    let table = Table::deserialize(&mut BufReader::new(File::open(&path)?))?;
    println!("Read table with {} rows from {path}.", table.len());

    let Some(first) = table.batches().first() else {
        println!("No columns.");
        return Ok(());
    };

    for (col, chunk) in first.columns.iter().enumerate() {
        match chunk.col_type() {
            ColumnType::Int => {
                println!("Column {col} (int): mean = {}", int_mean(&table, col));
            }
            ColumnType::Bool => {
                println!("Column {col} (bool): entropy = {}", bool_entropy(&table, col));
            }
            ColumnType::String => {
                let counts = ascii_counts(&table, col);
                let total: u64 = counts.iter().sum();
                println!("Column {col} (varchar): {total} ASCII characters");
                for (c, &n) in counts.iter().enumerate().filter(|&(_, &n)| n > 0) {
                    println!("    {:?}: {n}", c as u8 as char);
                }
            }
        }
    }

    Ok(())
}

fn int_mean(table: &Table, col: usize) -> i64 {
    let mut sum = 0i64;
    for batch in table.batches() {
        if let ColumnChunk::Int(v) = &batch.columns[col] {
            sum += v.iter().sum::<i64>();
        }
    }
    sum / table.len() as i64
}

fn bool_entropy(table: &Table, col: usize) -> f64 {
    let mut trues = 0usize;
    for batch in table.batches() {
        if let ColumnChunk::Bool(v) = &batch.columns[col] {
            trues += v.iter().filter(|&&b| b).count();
        }
    }

    let p = trues as f64 / table.len() as f64;
    [p, 1.0 - p]
        .into_iter()
        .filter(|&x| x > 0.0)
        .map(|x| -x * x.log2())
        .sum()
}

fn ascii_counts(table: &Table, col: usize) -> [u64; 128] {
    let mut counts = [0u64; 128];
    for batch in table.batches() {
        if let ColumnChunk::String { data, .. } = &batch.columns[col] {
            for b in data.bytes().filter(u8::is_ascii) {
                counts[b as usize] += 1;
            }
        }
    }
    counts
}
