use std::fs::File;

use csv::Reader;

#[derive(Clone, PartialEq, Debug)]
pub enum DataVal{
    Int(i64),
    String(String),
    Float(f64),
}//end enum ColumnType

#[derive(Clone, PartialEq, Debug)]
pub struct DataCell {
    header: String,
    data: DataVal,
}//end struct DataLine

#[allow(dead_code)]
impl DataCell {
    /// Constructs a new DataLine
    pub fn new(header: &String, value: String) -> DataCell {
        // test if value is int
        match value.parse::<i64>() {
            Ok(i) => {
                DataCell {
                    header: header.to_owned(),
                    data: DataVal::Int(i)
                }//end struct construction
            },//end Ok int Case
            Err(_) => {
                // test if value is float
                match value.parse::<f64>() {
                    Ok(f) => {
                        DataCell {
                            header: header.to_owned(),
                            data: DataVal::Float(f),
                        }//end struct Construction
                    },// end Ok float Case
                    Err(_) => {
                        DataCell {
                            header: header.to_owned(),
                            data: DataVal::String(value),
                        }//end struct Construction
                    }//end Err float, must be str Case
                }//end matching if value is float
            },//end Err int, test float Case
        }//end matching if value is int
    }//end fn new()

    pub fn get_header(&self) -> &String {&self.header}
    pub fn get_data(&self) -> &DataVal {&self.data}
}

#[derive(Clone,PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
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

    pub fn get_headers(&self) -> Vec<&String> {
        let mut ref_vec = Vec::new();
        for header in &self.headers {ref_vec.push(header)}
        ref_vec
    }//end get_headers()
    pub fn get_headers_ref(&self) -> &Vec<String> {&self.headers}
    pub fn get_records(&self) -> Vec<&DataRow> {
        let mut ref_vec = Vec::new();
        for data_row in &self.records {ref_vec.push(data_row);}
        ref_vec
    }//end get_records()
    pub fn get_records_ref(&self) -> &Vec<DataRow> {&self.records}
    pub fn get_record(&self, row_idx: usize, col_idx: usize) -> Option<&DataCell> {
        if let Some(data_row) = self.records.get(row_idx) {
            data_row.get_data(col_idx)
        } else {None}
    }//end get_record()
}

/// Splits records up based on unique values in the specified column index.
/// So, for example, if a sample id has header index 0, and you have sample ids
/// of [1,2,3], then calling this function with col_splt_idx of 0 would give
/// a Vec of data rows which sample id 1, a Vec of with only sample id 2, etc.
pub fn get_split_records(records: &Vec<DataRow>, col_splt_idx: usize) -> Option<Vec<(&DataVal, Vec<&DataRow>)>> {
    let mut wrapping_vec: Vec<(&DataVal, Vec<&DataRow>)> = Vec::new();
    for record in records {
        if let Some(this_data_at_col) = record.get_data(col_splt_idx) {
            // test if this_data_at_col matches DataVals we've already recorded
            let this_data_val = this_data_at_col.get_data();
            let mut found_match = false;
            for (data_val, row_group) in &mut wrapping_vec.iter_mut() {
                if this_data_val == *data_val {
                    found_match = true;
                    row_group.push(record);
                }//end if we found a match
            }//end adding row with this_data_val/this_data_at_col to entry for matching
            if !found_match {
                let mut new_row_group = Vec::new();
                new_row_group.push(record);
                wrapping_vec.push((this_data_val, new_row_group));
            }//end if we need to add another group to wrapping vec
        } else {println!("Couldn't get data at col idx {} for row data {:?}", col_splt_idx, record.get_row_data())}
    }//end looping over all records

    Some(wrapping_vec)
}//end get_split_records()

/// Gets the average value from a single column of the grid made up of DataRows.
/// Returns avg of integer values found, number of strings found, and avg of floats found 
pub fn get_col_avg(records: &Vec<&DataRow>, col_idx: usize) -> (i64, f64, usize) {
    let mut running_sums: (i64, f64) = (0,0.0);
    // int, float, string
    let mut running_counts: (i64, f64, usize) = (0,0.0,0);
    for row in records {
        if let Some(this_cell_at_col) = row.get_data(col_idx) {
            match this_cell_at_col.data {
                DataVal::Int(i) => {running_sums.0 += i; running_counts.0 += 1;},
                DataVal::Float(f) => {running_sums.1 += f; running_counts.1 += 1.0;},
                DataVal::String(_) => {running_counts.2 += 1;},
            }//end matching type of cell data
        } else {println!("Couldn't get data at col idx {} for row data {:?}", col_idx, row.get_row_data())}
    }//end looping over each row
    let int_avg; if running_counts.0 != 0 {
        int_avg = running_sums.0 / running_counts.0;
    } else {int_avg = 0;}
    let flt_avg; if running_counts.1 != 0.0 {
        flt_avg = running_sums.1 / running_counts.1;
    } else {flt_avg = 0.0;}

    (int_avg, flt_avg, running_counts.2)
}//end get_col_avg