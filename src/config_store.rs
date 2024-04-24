
/// This struct is meant to store configuration inforamation
/// in a way that is not reliant on a specific ui implementation,
/// such that it can be passed around easily.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct ConfigStore {
    /// Tells whether or not we should be filtering csv
    /// data to only include rows with a specific classification.
    pub csv_class_filter_enabled: bool,
    /// If we're filtering input csv data to only include rows
    /// with a specific classification, this tells us what
    /// classification we're filtering for, such as "Sound".
    pub csv_class_filter_filters: Vec<String>,
    /// Tells us whether we should include columns in output
    /// that are essentially statistics about certain columns
    /// in the input csv.
    pub csv_stat_columns_enabled: bool,
    /// If we're including columns in the output that are essentially
    /// statistics about certain columns in the input csv, this tells
    /// us which columns in the input csv to do statistics on.
    pub csv_stat_columns_columns: Vec<String>,
    /// Tells us whether we should include columns in the output
    /// about what percentage of each sample has each classification.  
    /// So, %Sound, %Sorghum, etc.
    pub csv_class_percent_enabled: bool,
    /// Tells us whether we should include columns in the output
    /// that are pulled from sieve data in the xml file. If no
    /// xml file is loaded, then this is meaningless.
    pub xml_sieve_cols_enabled: bool,
}//end struct ConfigStore
