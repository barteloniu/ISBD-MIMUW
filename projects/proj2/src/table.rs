use std::iter::zip;

pub const BATCH_SIZE: usize = 1024;

// TODO proper constructors, less pub

// Just a bunch of batches with a guarantee that their types much.
#[derive(Debug, PartialEq)]
pub struct Table {
    len: usize,
    batches: Vec<Batch>,
}

#[derive(Debug)]
pub enum TableCreationError {
    ColumnTypeMismatch,
    BatchSizeMismatch,
}

impl Table {
    pub fn new(batches: Vec<Batch>) -> Result<Table, TableCreationError> {
        if batches.is_empty() {
            return Ok(Table {
                len: 0,
                batches: batches,
            });
        }

        // Check if all types match.
        let (first, rest) = batches.split_first().unwrap();
        if !rest.iter().all(|b| first.same_type(b)) {
            return Err(TableCreationError::ColumnTypeMismatch);
        }

        // Check if the last element is nonempty and all before it have BATCH_SIZE elems.
        let (last, full) = batches.split_last().unwrap();
        if last.len() == 0 || last.len() > BATCH_SIZE || full.iter().any(|b| b.len() != BATCH_SIZE)
        {
            return Err(TableCreationError::BatchSizeMismatch);
        }

        let len = full.len() * BATCH_SIZE + last.len();
        Ok(Table { len, batches })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn batches(&self) -> &[Batch] {
        self.batches.as_slice()
    }
}

#[derive(Debug, PartialEq)]
pub struct Batch {
    pub len: usize,
    pub columns: Vec<ColumnChunk>,
}

impl Batch {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get_int(&self, col: usize, row: usize) -> Option<i64> {
        self.columns.get(col)?.get_int(row)
    }

    pub fn get_bool(&self, col: usize, row: usize) -> Option<bool> {
        self.columns.get(col)?.get_bool(row)
    }

    pub fn get_string(&self, col: usize, row: usize) -> Option<&str> {
        self.columns.get(col)?.get_string(row)
    }

    fn same_type(&self, other: &Self) -> bool {
        self.columns.len() == other.columns.len()
            && zip(self.columns.iter(), other.columns.iter())
                .all(|(c1, c2)| c1.col_type() == c2.col_type())
    }
}

#[derive(Debug, PartialEq)]
pub enum ColumnChunk {
    Int(Vec<i64>),
    Bool(Vec<bool>),
    String { data: String, ends: Vec<usize> },
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Int,
    Bool,
    String,
}

impl TryFrom<u8> for ColumnType {
    type Error = u8;

    fn try_from(b: u8) -> Result<Self, Self::Error> {
        match b {
            x if x == ColumnType::Int as u8 => Ok(ColumnType::Int),
            x if x == ColumnType::Bool as u8 => Ok(ColumnType::Bool),
            x if x == ColumnType::String as u8 => Ok(ColumnType::String),
            _ => Err(b),
        }
    }
}

impl ColumnChunk {
    pub fn len(&self) -> usize {
        match self {
            ColumnChunk::Int(v) => v.len(),
            ColumnChunk::Bool(v) => v.len(),
            ColumnChunk::String { ends, .. } => ends.len(),
        }
    }

    pub fn get_int(&self, row: usize) -> Option<i64> {
        match self {
            ColumnChunk::Int(v) => v.get(row).copied(),
            _ => None,
        }
    }

    pub fn get_bool(&self, row: usize) -> Option<bool> {
        match self {
            ColumnChunk::Bool(v) => v.get(row).copied(),
            _ => None,
        }
    }

    pub fn get_string(&self, row: usize) -> Option<&str> {
        match self {
            ColumnChunk::String { data, ends } => {
                let end = *ends.get(row)?;
                let start = row.checked_sub(1).map_or(0, |i| ends[i]);

                Some(&data[start..end])
            }
            _ => None,
        }
    }

    pub fn col_type(&self) -> ColumnType {
        match self {
            ColumnChunk::Int(_) => ColumnType::Int,
            ColumnChunk::Bool(_) => ColumnType::Bool,
            ColumnChunk::String { .. } => ColumnType::String,
        }
    }
}
