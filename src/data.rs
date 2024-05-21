use std::{fs::File, io::BufReader};

use csv::Reader;
use quick_xml::events::Event;

/// Holds the value within a Cell, which might be a String, Int, or Float.
#[derive(Clone, PartialEq, Debug)]
pub enum DataVal{
    Int(i64),
    String(String),
    Float(f64),
}//end enum ColumnType

impl DataVal {
    /// Returns the string representation of the
    /// underlying data, using format!()
    pub fn to_string(&self) -> String {
        match self {
            DataVal::Int(i) => format!("{}", i),
            DataVal::String(s) => format!("{}", s),
            DataVal::Float(f) => format!("{}", f),
        }//end matching self
    }//end to_string()
}//end impl for DataVal
/// Represents an individual cell of data,
/// holding a copy of the header it's under.  
/// This struct is largely intended to be used by 
/// DataRow and Data structs.  
/// For more info, see documentation example for DataCell::new() and DataCell::new_from_val().
#[derive(Clone, PartialEq, Debug)]
pub struct DataCell {
    header: String,
    data: DataVal,
}//end struct DataLine

#[allow(dead_code)]
impl DataCell {
    /// Constructs a new DataLine, automatically creating
    /// the proper DataVal by parsing and testing value.  
    /// 
    /// # Examples
    /// 
    /// ```
    /// use usda_c_grain_sum::data::DataVal;
    /// use usda_c_grain_sum::data::DataCell;
    /// 
    /// let header = String::from("Length");
    /// let val_str = String::from("5.2");
    /// 
    /// let datacell = DataCell::new(&header, val_str);
    /// 
    /// assert_eq!(*datacell.get_data(), DataVal::Float(5.2));
    /// ```
    /// 
    /// ```
    /// use usda_c_grain_sum::data::DataVal;
    /// use usda_c_grain_sum::data::DataCell;
    /// 
    /// let header = String::from("Red");
    /// let val_str = String::from("55");
    /// 
    /// let datacell = DataCell::new(&header, val_str);
    /// 
    /// assert_eq!(*datacell.get_data(), DataVal::Int(55));
    /// ```
    /// 
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

    /// Constructs a DataCell from a DataVal object without needing to do String parsing.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use usda_c_grain_sum::data::DataVal;
    /// use usda_c_grain_sum::data::DataCell;
    /// 
    /// let header = String::from("Area");
    /// let dataval = DataVal::Float(5.4);
    /// 
    /// let datacell = DataCell::new_from_val(&header, dataval.clone());
    /// 
    /// assert_eq!(*datacell.get_data(), dataval);
    /// ```
    pub fn new_from_val(header: &String, value: DataVal) -> DataCell {
        DataCell {
            header: header.to_owned(),
            data: value,
        }//end struct construction
    }//end new_from_val(header, value)

    /// Gets reference to the header label of this cell.
    pub fn get_header(&self) -> &String {&self.header}
    /// Gets reference to the DataVal of this cell.
    pub fn get_data(&self) -> &DataVal {&self.data}
}


/// Represents a single row of data, with each cell in that row
/// being represented by a DataCell.  
/// Also holds a copy of the index of this Row in the larger Data struct.  
/// This struct is largely meant to be constructed by Data.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// 
/// // create the column headers
/// let header_0 = String::from("Area");
/// let header_1 = String::from("Length");
/// let header_2 = String::from("Width");
/// let header_3 = String::from("Thickness");
/// 
/// // create the cells to go in the row
/// let mut cell_vec: Vec<DataCell> = Vec::new();
/// cell_vec.push(DataCell::new_from_val(&header_0, DataVal::Float(5.4)));
/// cell_vec.push(DataCell::new_from_val(&header_1, DataVal::Float(3.7)));
/// cell_vec.push(DataCell::new_from_val(&header_2, DataVal::Float(1.5)));
/// cell_vec.push(DataCell::new_from_val(&header_3, DataVal::Float(1.3)));
/// 
/// // create the row itself
/// let row_index = 0;
/// let datarow = DataRow::new(row_index, cell_vec.clone());
/// 
/// // test the values
/// assert_eq!(*datarow.get_row_idx(), 0);
/// assert_eq!(*datarow.get_row_data(), cell_vec);
/// assert_eq!(*datarow.get_data(0).unwrap().get_data(), DataVal::Float(5.4));
/// assert_eq!(*datarow.get_data(1).unwrap().get_data(), DataVal::Float(3.7));
/// assert_eq!(*datarow.get_data(2).unwrap().get_data(), DataVal::Float(1.5));
/// assert_eq!(*datarow.get_data(3).unwrap().get_data(), DataVal::Float(1.3));
/// assert_eq!(datarow.get_data(4), None);
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct DataRow {
    row_idx: usize,
    row_data: Vec<DataCell>,
}//end struct DataRow

#[allow(dead_code)]
impl DataRow {
    /// Constructs a new DataRow
    pub fn new(row_idx: usize, row_data: Vec<DataCell>) -> DataRow {
        DataRow {
            row_idx,
            row_data,
        }//end struct construction
    }//end new()

    /// Gets reference to row index in of this DataRow.
    pub fn get_row_idx(&self) -> &usize {&self.row_idx}
    /// Gets Reference to the vector of DataCells contained in this struct.
    pub fn get_row_data(&self) -> &Vec<DataCell> {&self.row_data}
    /// Returns a reference to the particular DataCell if the index is valid.  
    /// If the index is out of bounds, returns None.
    pub fn get_data(&self, idx: usize) -> Option<&DataCell> {self.row_data.get(idx)}
}//end impl for DataRow

/// Holds all the data from one csv/xlsx file.  
/// Uses something like "Parse, don't Validate" to ensure
/// data is accurate to the file.  
/// Because of this, it is intended for processing functions to 
/// run filters and processing on references to the records 
/// contained here instead of mutating the struct itself.  
/// The component structs, DataRow and DataCell, also
/// reflect this design.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// use usda_c_grain_sum::data::Data;
/// 
/// // create the column headers
/// let mut column_headers: Vec<String> = Vec::new();
/// let header_0 = String::from("Length");
/// let header_1 = String::from("Width");
/// let header_2 = String::from("Thickness");
/// column_headers.push(header_0.clone());
/// column_headers.push(header_1.clone());
/// column_headers.push(header_2.clone());
/// 
/// // create vectors of DataCells, to be made into DataRows
/// let mut cell_row_0: Vec<DataCell> = Vec::new();
/// cell_row_0.push(DataCell::new_from_val(&header_0, DataVal::Float(5.4)));
/// cell_row_0.push(DataCell::new_from_val(&header_1, DataVal::Float(3.2)));
/// cell_row_0.push(DataCell::new_from_val(&header_2, DataVal::Float(2.1)));
/// let mut cell_row_1: Vec<DataCell> = Vec::new();
/// cell_row_1.push(DataCell::new_from_val(&header_0, DataVal::Float(6.7)));
/// cell_row_1.push(DataCell::new_from_val(&header_1, DataVal::Float(4.5)));
/// cell_row_1.push(DataCell::new_from_val(&header_2, DataVal::Float(2.9)));
/// 
/// // construct the two DataRows and add them to a Vec
/// let mut datarow_vec: Vec<DataRow> = Vec::new();
/// let datarow_0 = DataRow::new(0, cell_row_0);
/// let datarow_1 = DataRow::new(1, cell_row_1);
/// datarow_vec.push(datarow_0);
/// datarow_vec.push(datarow_1);
/// 
/// // construct the Data struct
/// let data = Data::from_row_data(column_headers.clone(), datarow_vec.clone());
/// 
/// // test that headers are correct
/// assert_eq!(*data.get_headers_ref(), column_headers);
/// assert_eq!(data.get_header_index("Length").unwrap(), 0);
/// assert_eq!(data.get_header_index("Width").unwrap(), 1);
/// assert_eq!(data.get_header_index("Thickness").unwrap(), 2);
/// assert_eq!(*data.get_header_from_index(0).unwrap(), header_0);
/// assert_eq!(*data.get_header_from_index(1).unwrap(), header_1);
/// assert_eq!(*data.get_header_from_index(2).unwrap(), header_2);
/// 
/// // test that records are correct
/// assert_eq!(*data.get_records_ref(), datarow_vec);
/// assert_eq!(*data.get_record(0,0).unwrap().get_data(), DataVal::Float(5.4));
/// assert_eq!(*data.get_record(0,1).unwrap().get_data(), DataVal::Float(3.2));
/// assert_eq!(*data.get_record(0,2).unwrap().get_data(), DataVal::Float(2.1));
/// assert_eq!(*data.get_record(1,0).unwrap().get_data(), DataVal::Float(6.7));
/// assert_eq!(*data.get_record(1,1).unwrap().get_data(), DataVal::Float(4.5));
/// assert_eq!(*data.get_record(1,2).unwrap().get_data(), DataVal::Float(2.9));
/// ```
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

    /// Reads data in from an xml file.
    pub fn from_xml_reader(mut reader: quick_xml::Reader<BufReader<File>>) -> Result<Data,String> {
        let mut buf = Vec::new();

        let mut data_rows: Vec<DataRow> = Vec::new();
        let mut data_cells: Vec<DataCell> = Vec::new();

        let sample_id_tag = b"sample-id";
        let external_id_tag = b"reference";
        let sample_start_end_tag = b"sample-result";
        // let sample_info_tags = vec![b"machine-id",b"sample-id",b"reference"];
        let sieving_starts_with = b"filter-sieving";
        let mut most_recent_tag = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => return Err(format!("Encountered error at position {}: {:?}",reader.buffer_position(),e)),
                Ok(Event::Eof) => break,

                Ok(Event::Start(byte_start)) => {
                    let tag_name = String::from_utf8(byte_start.name().as_ref().to_vec()).unwrap();//format!("{}",byte_start.name().as_ref());
                    if tag_name.as_bytes().eq(sample_id_tag) || tag_name.as_bytes().eq(external_id_tag) {
                        most_recent_tag = Some(tag_name);
                    } else if tag_name.as_bytes().starts_with(sieving_starts_with) {
                        most_recent_tag = Some(tag_name);
                    }//end cases of tag being relevant
                    // println!("attributes values: {:?}",byte_start.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>());
                }, //end start event case
                Ok(Event::Text(btxt)) => {
                    if most_recent_tag.is_some() {
                        let cur_tag = most_recent_tag.unwrap();
                        let txt = btxt.unescape().unwrap().into_owned();
                        let data_cell = DataCell::new(&cur_tag, txt);
                        data_cells.push(data_cell);
                        most_recent_tag = Some(cur_tag);
                    }//end if we have a recent tag
                },
                Ok(Event::End(bytes_end)) => {
                    // assume tag is either end of sample, valid for most recent, or not important
                    if bytes_end.name().as_ref().eq(sample_start_end_tag) {
                        let datarow = DataRow::new(data_rows.len(),data_cells.clone());
                        data_cells.clear();
                        data_rows.push(datarow);
                    }//end if this is the end of a sample
                    else if most_recent_tag.is_some() {
                        let m_r_t = most_recent_tag.unwrap();
                        // check for current tag closing, updating most_recent
                        if m_r_t.as_bytes().eq(bytes_end.name().as_ref()) {
                            most_recent_tag = None;
                        } else {most_recent_tag = Some(m_r_t);}
                    }//end if there was a recent tag
                    // println!("closing tag {}", String::from_utf8_lossy(bytes_end.name().as_ref()));
                },

                Ok(event) => println!("Unhandled event {:?}",event),
            }//end matching reader events
            buf.clear();
        }//end looping while we have stuff to read from file

        /*
        Instead of checking for new headers on every sample (within the loop),
        we'll assume that each sample has the same headers, and can thus be gathered
        by looking at each DataCell in the first DataRow.
         */
        let mut headers_vec = Vec::new();
        if let Some(first_row) = data_rows.first() {
            for data_cell in first_row.get_row_data() {
                let this_header = data_cell.get_header();
                if !headers_vec.contains(this_header) {
                    headers_vec.push(data_cell.get_header().clone());
                }//end if we found a new header
            }//end looping over each cell in first row
        }//end if we have at least one row

        let data = Data::from_row_data(headers_vec, data_rows);
        return Ok(data);
    }//end from_xml_reader(reader)

    /// Constructs a Data struct from a vector of headers and DataRows.
    /// 
    /// 
    pub fn from_row_data(headers: Vec<String>, row_data: Vec<DataRow>) -> Data {
        Data {
            headers,
            records: row_data,
        }//end struct construction
    }//end from_row_data(headers, row_data)

    /// Returns vector of references to headers in struct.
    pub fn get_headers(&self) -> Vec<&String> {
        let mut ref_vec = Vec::new();
        for header in &self.headers {ref_vec.push(header)}
        ref_vec
    }//end get_headers()
    /// Returns reference to vector containing list of headers.
    pub fn get_headers_ref(&self) -> &Vec<String> {&self.headers}
    /// Finds the first index of the header specified.  
    /// If the header is not found, returns None.
    pub fn get_header_index(&self, target_header: &str) -> Option<usize> {
        for (i, header) in self.headers.iter().enumerate() {
            if header.eq(target_header) {
                return Some(i);
            }//end if we found a match
        }//end checking headers for match to header
        return None;
    }//end get_header_index(self, target_header)
    /// Gets the header at the specified index, if that index exists.  
    /// If the index is out of bounds, returns None.
    pub fn get_header_from_index(&self, index: usize) -> Option<&String> { self.headers.get(index) }
    /// Gets vector of references to DataRows in this struct.
    pub fn get_records(&self) -> Vec<&DataRow> {
        let mut ref_vec = Vec::new();
        for data_row in &self.records {ref_vec.push(data_row);}
        ref_vec
    }//end get_records()
    /// Gets a reference to the vector of DataRows in this struct.
    pub fn get_records_ref(&self) -> &Vec<DataRow> {&self.records}
    /// Gets a specific record at a given row and column index, returning 
    /// a reference to the DataCell there if the bounds are valid.  
    /// If the row or column index are not valid, returns None
    pub fn get_record(&self, row_idx: usize, col_idx: usize) -> Option<&DataCell> {
        if let Some(data_row) = self.records.get(row_idx) {
            data_row.get_data(col_idx)
        } else {None}
    }//end get_record()
}

/// Splits records up based on unique values in the specified column index.
/// So, for example, if a sample id has header index 0, and you have sample ids
/// of \[1,2,3\], then calling this function with col_splt_idx of 0 would give
/// a Vec of data rows which sample id 1, a Vec of with only sample id 2, etc.
/// 
/// # Errors
/// 
/// This function will return Err() if it cannot get a DataCell at some index.  
/// The Err String will contain information on which DataCell couldn't be accessed.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// use usda_c_grain_sum::data::Data;
/// use usda_c_grain_sum::data::get_split_records;
/// 
/// // set up headers
/// let mut column_headers: Vec<String> = Vec::new();
/// let header_0 = String::from("Class");
/// let header_1 = String::from("Volume");
/// column_headers.push(header_0.clone());
/// column_headers.push(header_1.clone());
/// 
/// // set up rows of DataCells
/// let mut cell_row_0: Vec<DataCell> = Vec::new();
/// let mut cell_row_1: Vec<DataCell> = Vec::new();
/// let mut cell_row_2: Vec<DataCell> = Vec::new();
/// cell_row_0.push(DataCell::new_from_val(&header_0, DataVal::String("Sorghum".to_string())));
/// cell_row_0.push(DataCell::new_from_val(&header_1, DataVal::Float(3.2)));
/// cell_row_1.push(DataCell::new_from_val(&header_0, DataVal::String("Sound".to_string())));
/// cell_row_1.push(DataCell::new_from_val(&header_1, DataVal::Float(6.7)));
/// cell_row_2.push(DataCell::new_from_val(&header_0, DataVal::String("Sorghum".to_string())));
/// cell_row_2.push(DataCell::new_from_val(&header_1, DataVal::Float(2.9)));
/// 
/// // set up DataRows and add them to vec
/// let mut datarow_vec: Vec<DataRow> = Vec::new();
/// let datarow_0 = DataRow::new(0, cell_row_0);
/// let datarow_1 = DataRow::new(1, cell_row_1);
/// let datarow_2 = DataRow::new(2, cell_row_2);
/// datarow_vec.push(datarow_0.clone());
/// datarow_vec.push(datarow_1.clone());
/// datarow_vec.push(datarow_2.clone());
/// 
/// // create the Data struct from everything
/// let data = Data::from_row_data(column_headers, datarow_vec);
/// 
/// let base_records: Vec<&DataRow> = data.get_records();
/// let class_split_records = get_split_records(&base_records, 0).unwrap();
/// let class_split_records_first: &(&DataVal, Vec<&DataRow>) = class_split_records.get(0).unwrap();
/// let class_split_records_second: &(&DataVal, Vec<&DataRow>) = class_split_records.get(1).unwrap();
/// 
/// assert_eq!(*class_split_records_first.0, DataVal::String("Sorghum".to_string()));
/// assert_eq!(class_split_records_first.1.len(), 2);
/// assert_eq!(*class_split_records_second.0, DataVal::String("Sound".to_string()));
/// assert_eq!(class_split_records_second.1.len(), 1);
/// ```
pub fn get_split_records<'a>(records: &'a Vec<&'a DataRow>, col_splt_idx: usize) -> Result<Vec<(&'a DataVal, Vec<&'a DataRow>)>, String> {
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
                let mut new_row_group: Vec<&DataRow> = Vec::new();
                new_row_group.push(record);
                wrapping_vec.push((this_data_val, new_row_group));
            }//end if we need to add another group to wrapping vec
        } else { return Err(format!("Couldn't get DataCell at col idx {} and row idx {} for row data {:?}", col_splt_idx, record.get_row_idx(), record.get_row_data())); }
    }//end looping over all records

    return Ok(wrapping_vec);
}//end get_split_records()

/// This function returns a Vector only containing DataRows 
/// whose value at column col_idx is equal to the expected.  
/// The intended purpose of this function is to return rows 
/// belonging to a particular category.  
/// For example, if you have a column 2 with header type, you 
/// might want to get rows with a type matching Sound.  
/// Notably, this function does not change the values of 
/// any DataRows or clone anything.  
/// Instead, the function reorganizes a list of references 
/// by returning a new list that only contains references 
/// to DataRows with the specified DataVal.  
/// 
/// # Errors
/// 
/// If the given column index is invalid, this function will 
/// return an error.
/// An error will also be returned if the function cannot access a 
/// particular row for some reason.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// use usda_c_grain_sum::data::Data;
/// use usda_c_grain_sum::data::get_filtered_records;
/// 
/// // set up headers
/// let mut column_headers: Vec<String> = Vec::new();
/// let header_0 = String::from("Length");
/// let header_1 = String::from("Width");
/// let header_2 = String::from("Thickness");
/// column_headers.push(header_0.clone());
/// column_headers.push(header_1.clone());
/// column_headers.push(header_2.clone());
/// 
/// // set up rows of DataCells
/// let mut cell_row_0: Vec<DataCell> = Vec::new();
/// let mut cell_row_1: Vec<DataCell> = Vec::new();
/// cell_row_0.push(DataCell::new_from_val(&header_0, DataVal::Float(5.4)));
/// cell_row_0.push(DataCell::new_from_val(&header_1, DataVal::Float(3.2)));
/// cell_row_0.push(DataCell::new_from_val(&header_2, DataVal::Float(2.1)));
/// cell_row_1.push(DataCell::new_from_val(&header_0, DataVal::Float(6.7)));
/// cell_row_1.push(DataCell::new_from_val(&header_1, DataVal::Float(4.5)));
/// cell_row_1.push(DataCell::new_from_val(&header_2, DataVal::Float(2.9)));
/// 
/// // set up DataRows and add them to vec
/// let mut datarow_vec: Vec<DataRow> = Vec::new();
/// let datarow_0 = DataRow::new(0, cell_row_0);
/// let datarow_1 = DataRow::new(1, cell_row_1);
/// datarow_vec.push(datarow_0.clone());
/// datarow_vec.push(datarow_1.clone());
/// 
/// // create the Data struct from everything
/// let data = Data::from_row_data(column_headers, datarow_vec);
/// 
/// assert_eq!(data.get_records().len(), 2);
/// 
/// let data_records: Vec<&DataRow> = data.get_records();
/// let length_5_4_filtered = get_filtered_records(&data_records, 0, DataVal::Float(5.4)).unwrap();
/// 
/// assert_eq!(length_5_4_filtered.len(), 1);
/// 
/// let first_row_first_col_data_cell: &DataCell = length_5_4_filtered
///     .first().unwrap() // get first (and only) DataRow from Vec
///     .get_data(0).unwrap(); // get first DataCell from first DataRow
/// 
/// assert_eq!(*first_row_first_col_data_cell.get_data(), DataVal::Float(5.4));
/// ```
/// 
pub fn get_filtered_records<'a>(records: &'a Vec<&'a DataRow>, col_idx: usize, expected: DataVal) -> Result<Vec<&'a DataRow>, String> {
    let mut filtered_vec: Vec<&DataRow> = Vec::new();

    for row in records {
        match row.get_data(col_idx) {
            Some(data_cell) => {
                // make sure data cell is equal to expected
                if expected == *data_cell.get_data() {
                    filtered_vec.push(row);
                }//end if this row matches the filter
            } None => return Err(format!("Couldn't get DataCell at col idx {} and row idx {} for row data {:?}, ", col_idx, row.get_row_idx(), row.get_row_data())),
        }//end matching whether we could access row data at col_idx
    }//end looping over each row

    Ok(filtered_vec)
}//end get_filtered_records()
