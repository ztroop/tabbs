//! tabbs is a command line tool to print comma-separated data as a table.
//!
//! Example usage:
//!
//! ```sh
//! echo "jack,35,neat\n\
//!       jane,50,cool\n\
//!       erin,20,ah" | tabb -c "name,age,text"
//! ```
//!
//! This will produce the following output:
//!
//! ```plaintext
//! +------+-----+------+
//! | name | age | text |
//! +------+-----+------+
//! | jack | 35  | neat |
//! | jane | 50  | cool |
//! | erin | 20  | nice |
//! +------+-----+------+
//! ```

use colored::*;
use std::env;
use std::io::{self, Read, Write};
use std::process;

/// The main function reads the command line arguments and standard input,
/// then calls the `print_table_to_writer` function to print the table to stdout.
///
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-c" {
        eprintln!(
            "Usage: tabb -c \"column1,column2,...\" [--header-color COLOR] [--cell-color COLOR]"
        );
        process::exit(1);
    }

    let column_names: Vec<&str> = args[2].split(',').collect();

    let header_color = args
        .iter()
        .position(|arg| arg == "--header-color")
        .and_then(|pos| args.get(pos + 1).map(|s| s.to_owned()));
    let cell_color = args
        .iter()
        .position(|arg| arg == "--cell-color")
        .and_then(|pos| args.get(pos + 1).map(|s| s.to_owned()));

    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_to_string(&mut input)
        .expect("Failed to read input");

    let rows: Vec<Vec<String>> = input
        .split_whitespace()
        .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
        .collect();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    print_table_to_writer(
        &column_names,
        &rows,
        header_color.as_deref(),
        cell_color.as_deref(),
        &mut handle,
    );
}

/// Print a table with the given column names, row data, optional header and cell colors to the provided writer.
///
/// # Arguments
///
/// * `column_names` - A slice of strings representing the column names.
/// * `rows` - A slice of Vec<String> representing the rows of data.
/// * `header_color` - An optional string specifying the color of the header text.
/// * `cell_color` - An optional string specifying the color of the cell text.
/// * `writer` - A mutable reference to a writer implementing the `Write` trait.
///
fn print_table_to_writer(
    column_names: &[&str],
    rows: &[Vec<String>],
    header_color: Option<&str>,
    cell_color: Option<&str>,
    writer: &mut impl Write,
) {
    let mut column_widths: Vec<usize> = column_names.iter().map(|s| s.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                column_widths[i] = column_widths[i].max(cell.len());
            }
        }
    }

    let separator: String = column_widths
        .iter()
        .map(|width| "-".repeat(width + 2))
        .collect::<Vec<String>>()
        .join("+");

    writeln!(writer, "+{}+", separator).unwrap();
    write!(writer, "|").unwrap();
    for (i, column_name) in column_names.iter().enumerate() {
        let colored_column_name = header_color.map_or(column_name.to_string(), |color| {
            column_name.color(color).to_string()
        });
        write!(
            writer,
            " {:<width$} |",
            colored_column_name,
            width = column_widths[i]
        )
        .unwrap();
    }
    writeln!(writer).unwrap();
    writeln!(writer, "+{}+", separator).unwrap();

    for row in rows {
        write!(writer, "|").unwrap();
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                let colored_cell =
                    cell_color.map_or(cell.to_string(), |color| cell.color(color).to_string());
                write!(
                    writer,
                    " {:<width$} |",
                    colored_cell,
                    width = column_widths[i]
                )
                .unwrap();
            }
        }
        writeln!(writer).unwrap();
    }

    writeln!(writer, "+{}+", separator).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_to_writer() {
        let column_names = ["name", "age", "text"];
        let rows = [
            vec!["jack".to_string(), "35".to_string(), "neat".to_string()],
            vec!["jane".to_string(), "50".to_string(), "cool".to_string()],
            vec!["erin".to_string(), "20".to_string(), "nice".to_string()],
        ];

        let expected_output = "\
+------+-----+------+
| name | age | text |
+------+-----+------+
| jack | 35  | neat |
| jane | 50  | cool |
| erin | 20  | nice |
+------+-----+------+";
        let mut output = Vec::new();

        {
            let mut output_writer = std::io::BufWriter::new(output.by_ref());
            print_table_to_writer(&column_names, &rows, None, None, &mut output_writer);
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }
}
