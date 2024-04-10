use std::fs::File;

use csv::Reader;

#[derive(Clone, PartialEq)]
pub enum DataVal{
    Int(i32),
    String(String),
    Float(f32),
}//end enum ColumnType

#[derive(Clone, PartialEq)]
pub struct DataCell {
    header: String,
    data: DataVal,
}//end struct DataLine

#[allow(dead_code)]
impl DataCell {
    /// Constructs a new DataLine
    pub fn new(header: &String, value: String) -> DataCell {
        // test if value is float
        match value.parse::<f32>() {
            Ok(f) => {
                DataCell {
                    header: header.to_owned(),
                    data: DataVal::Float(f)
                }//end struct construction
            },//end Ok float Case
            Err(_) => {
                // test if value is int
                match value.parse::<i32>() {
                    Ok(i) => {
                        DataCell {
                            header: header.to_owned(),
                            data: DataVal::Int(i),
                        }//end struct Construction
                    },// end Ok int Case
                    Err(_) => {
                        DataCell {
                            header: header.to_owned(),
                            data: DataVal::String(value),
                        }//end struct Construction
                    }//end Err int, must be str Case
                }//end matching is value is int
            },//end Err float, test int Case
        }//end matching if value is float
    }//end fn new()

    pub fn get_header(&self) -> &String {&self.header}
    pub fn get_data(&self) -> &DataVal {&self.data}
}

#[derive(Clone,PartialEq)]
pub struct DataRow {
    row_idx: usize,
    row_data: Vec<DataCell>,
}//end struct DataRow

#[allow(dead_code)]
impl DataRow {
    pub fn new(row_idx: usize, row_data: Vec<DataCell>) -> DataRow {
        DataRow {
            row_idx,
            row_data,
        }//end struct construction
    }//end new()

    pub fn get_row_idx(&self) -> &usize {&self.row_idx}
    pub fn get_row_data(&self) -> &Vec<DataCell> {&self.row_data}
    pub fn get_data(&self, idx: usize) -> Option<&DataCell> {self.row_data.get(idx)}
}//end impl for DataRow

#[derive(Clone)]
pub struct Data {
    headers: Vec<String>,
    records: Vec<DataRow>,
}//end struct Data

#[allow(dead_code)]
impl Data {
    /// Reads all csv info into Data struct from reader.
    /// Todo: Maybe look into csvs_convert crate to convert to database for storage/speed
    pub fn from_csv_reader(mut reader: Reader<File>) -> Option<Data> {
        if let Ok(header_recs) = reader.headers() {
            let mut headers: Vec<String> = Vec::new();
            for header in header_recs {
                headers.push(header.to_string());
            }//end adding all headers to our vec
            
            let mut data_records: Vec<DataRow> = Vec::new();

            // Parse records from everything in the csvs
            for (row_idx, row_str) in reader.records().enumerate() {
                match row_str {
                    Ok(row_record) => {
                        // row_record is format of StringRecord(["893", "202403190019", "23GRY_DTD_264"...])
                        let mut tmp_row_data = Vec::new();
                        for (col_idx, cell_str) in row_record.into_iter().enumerate() {
                            match headers.get(col_idx) {
                                Some(header) => {
                                    let new_data_cell = DataCell::new(header, cell_str.to_string());
                                    tmp_row_data.push(new_data_cell);
                                },
                                None => {println!("Couldn't find header at col index {}. We have {} headers.", col_idx, headers.len())}
                            }//end matching whether we can get the header
                        }//end looping over each cell in this row
                        // add this whole row of data as a new DataRow
                        let new_data_row = DataRow::new(row_idx, tmp_row_data);
                        data_records.push(new_data_row);
                    },
                    Err(error) => println!("{}", error),
                }//end matching whether we got this row correctly
            }//end looping over each non-header record/row in csv
            return Some( Data {headers, records: data_records} );
        } else { return None; }
    }//end from_csv_reader()

    pub fn get_headers(&self) -> &Vec<String> {&self.headers}
    pub fn get_records(&self) -> &Vec<DataRow> {&self.records}
    pub fn get_record(&self, row_idx: usize, col_idx: usize) -> Option<&DataCell> {
        if let Some(data_row) = self.records.get(row_idx) {
            data_row.get_data(col_idx)
        } else {None}
    }//end get_record()
}