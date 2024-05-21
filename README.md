# C-Grain Summarizer

This program is intended to be used with output files from the C-Grain machine in order to generate useful summary files.
Files from the C-Grain can be output as csv or xml files. When documentation in this file or elsewhere refers to "input files", it is these csv or xml files that are being referred to.

Since this is a cargo project, simply use `cargo run` to compile and run the program from the same directory as the cargo.toml file.

Automated tests can be executed with `cargo test`. To build a release version, use `cargo run --release` or `cargo build --release`. Documentation can be generated in the target folder using `cargo doc`. For more information on cargo commands, see the cargo documentation.

The cargo.toml file can be read to find additional package information, such as the version of this package, the version of rust this package compiles with, and all dependencies used, along with their versions.

## Application Structure

In general, this project has something like an MVC architecture.

### View

The View is the gui module, contained in the gui.rs file. All of the widgets are initially set up in the `initialize()` method, similar to an `initialize_components` method in Java or C#. File dialog is handled by setting callbacks for click events, and the Sender Receiver pair is used to send messages to the main function.

### Controller

The Controller is the main module, and the main application loop is found in the main function of the main.rs file.
For the most part, the main module just listens for messages while the GUI is running, responding to them as they come.
Since the messages are passed as an enum, `InterfaceMessage`, it is simple to see a list of all possible valid messages that might be passed.

### Model

The Model is represented by several modules. You can easily tell which modules are Models because their modules are defined in lib.rs instead of main. These include:

- config_store: This module contains a struct called ConfigStore, which stored all the configuration information that gets saved and read from a file. It also contains functions to handle the File I/O of reading and writing from the config file. Serialization and Deserialization is handled by Serde.
- data: This module contains several structs and an enum with the primary purpose of storing data read in from various files. It also has a couple functions, `get_split_records()` and `get_filtered_records()`, which can be used to sort or group a vector of DataRows, such as you might receive from a Data object. The structs are details below:
  - `DataVal`: This enum represents the value within a single value. Since our input contains a mixture of Strings, Floats, and Integers, the DataVal enum was created to store any input value in one type and then pattern match when necessary.
  - `DataCell`: This enum represents a single cell within a table. Thus, it has a single value within it. It also has a method which can create a DataCell from a String, allowing it to partially handle deserialization of input data. Each DataCell also stores the name of the header it was under in the input, as a String.
  - `DataRow`: This enum represents a single row of cells within a table. It has a vector of DataCells, and it also stores a row index, which is supposed to indicate its location within the input data.
  - `Data`: This enum represents all of the data read from an input file. It contains a vector of DataRows, and it has functions to create a Data object from a csv or xml reader, allowing it to handle deserialization of input files. It also stores a vector of all the headers found in an input file, which is more accurate for csv inputs than xml.

- process: This module contains a number of functions which process data into another form, do calculations, along with a few functions used for saving processed data to an output file. It also contains the SampleOutput struct, which is simply a shorthand for data that has already been processed and is ready to be written to an excel sheet.
  - `proc_csv_stat_cols()` `proc_csv_class_per()` `proc_xml_sieve_data()`: Used for converting Data from input file into SampleOutputs with various information. If the operation fails for some reason, returns an error message as a String.
  - `get_workbook(output_path: &PathBuf) -> Result<Workbook,String>` `close_workbook()` `write_output_to_sheet()`: These functions all deal with excel files. They are used for opening up a file, saving SampleOutput to a new sheet in the file, and closing the file.
  - `get_sum_count()` `get_col_avg()` `get_col_stdev()`: These functions computer the sums, counts, averages, and standard deviations on a column within a vector of DataRows. They all use Result types to return an error String if they fail. One thing shared by these functions is that they do not attempt to merge DataVals with different types, so they will do separate calculations for Strings, Floats, and Integers.
  - `get_col_avg_sngl()` `get_col_stdev_sngl()`: These functions are similar to `get_col_avg()` and `get_col_stdev()`, but they differ in that they attempt to merge an Integer and Float values together to get a calculation over all numeric values in a column.
  - `precision_f64`: This function truncates decimal places from a float. This is used to limit the number of decimal places written to an excel file without needing to convert it to a string and back again. This function is only useful due to some of the limitations of the `simple_excel_writer` crate, which is used to write output files.
