use serde::{Deserialize, Serialize};
use std::{env, fs::{self, File}, io::Write, path::PathBuf};

/// This struct is meant to store configuration inforamation
/// in a way that is not reliant on a specific ui implementation,
/// such that it can be passed around easily.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Deserialize, Serialize)]
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
    /// Gives potential information on whether this config is
    /// personalized for a particular person.  
    /// The handling for this is likely to be kinda jank.
    pub personalized_config_name: String
}//end struct ConfigStore

impl Default for ConfigStore {
    fn default() -> Self {
        let class_filters_vec = vec!["Sound"];
        let stat_columns_vec = vec!["Area","Length","Width","Thickness","Ratio","Mean Width","Volume","Weight","Light","Hue","Saturation","Red","Green","Blue"];
        Self {
            csv_class_filter_enabled: true,
            csv_class_filter_filters: class_filters_vec.into_iter().map(|elem| elem.to_string()).collect(),
            csv_stat_columns_enabled: true,
            csv_stat_columns_columns: stat_columns_vec.into_iter().map(|elem| elem.to_string()).collect(),
            csv_class_percent_enabled: true,
            xml_sieve_cols_enabled: true,
            personalized_config_name: "".to_string(),
        }//end struct initialization
    }//end default()
}//end impl Default for ConfigStore

/// Gets default config which is personalized for needs of Scott
pub fn get_scott_config() -> ConfigStore {
    let mut conf = ConfigStore::default();
    conf.personalized_config_name = String::from("Scott");
    return conf;
}//end get_scott_config()

/// Gets default config which is personalized for needs of Rhett
pub fn get_rhett_config() -> ConfigStore {
    let mut conf = ConfigStore::default();
    conf.personalized_config_name = String::from("Rhett");
    return conf;
}//end get_rhett_config()

/// Attempts to determine the path to the config file.  
/// Assumes that config file has filename of [config_name] and extension of .config.  
/// If [create_if_missing] is true, and the file at path does not exist, then it will be created with default values.  
/// If [create_if_missing] is false, then this function does not check whether or not the filepath exists.
pub fn try_read_config_path(config_name: &str, create_if_missing: bool) -> Result<PathBuf, String> {
    // directory which contains exe this program runs from
    let exe_path = {
        match env::current_exe() {
            Ok(exe_path) => exe_path,
            Err(error) => return Err(error.to_string()),
        }//end matching whether we could get the current exe path
    };

    // set config path to be same parent as exe_path, but config_name
    let config_path = {
        let mut config_path = exe_path.clone();
        config_path.set_file_name(config_name);
        config_path.set_extension("config");
        config_path
    };

    // depending on parameter, ensure config file exists
    if !config_path.exists() && create_if_missing {
        match File::create(config_path.clone()) {
            Ok(mut file) => {
                let default_config = ConfigStore::default();
                match serde_json::to_string(&default_config) {
                    Ok(serialized_config) => {
                        match file.write_all(serialized_config.as_bytes()) {
                            Ok(_) => (),
                            Err(error) => return Err(error.to_string()),
                        }//end matching whether file write was successful
                    },
                    Err(error) => return Err(error.to_string()),
                }//end matching whether or not serde serialization worked
            },
            Err(error) => return Err(error.to_string()),
        }//end matching if file was created
    }//end if config_path does not exist

    Ok(config_path)
}//end try_read_config_path()

/// Attempts to read contents of file at path and deserialize into ConfigStore object.
pub fn try_read_config(config_path: &PathBuf) -> Result<ConfigStore,String> {
    match fs::read_to_string(config_path) {
        Ok(file_contents) => {
            match serde_json::from_str(&file_contents) {
                Ok(config_store) => Ok(config_store),
                Err(error) => Err(error.to_string()),
            }//end matching whether we can deserialize config
        },
        Err(error) => Err(error.to_string())
    }//end matching whether we could read string from file
}//end try_read_config()

/// Attempts to write given config_store to the given path.
pub fn try_write_config(config_path: &PathBuf, config_store: &ConfigStore) -> Result<(),String> {
    match File::create(config_path) {
        Ok(mut file) => {
            match serde_json::to_string(config_store) {
                Ok(config_serial) => {
                    match file.write_all(config_serial.as_bytes()) {
                        Ok(_) => Ok(()),
                        Err(error) => Err(error.to_string()),
                    }//end matching whether or not write succeeded
                },
                Err(error) => Err(error.to_string()),
            }//end matching whether we could serialize config
        },
        Err(error) => Err(error.to_string()),
    }//end matching whether we can see the file
}//end try_write_config()