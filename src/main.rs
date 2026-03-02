//! tabbs is a command line tool to print delimiter-separated data as a table.
//!
//! Example usage:
//!
//! ```sh
//! printf "jack,35,neat\njane,50,cool\nerin,20,nice" | tabbs -c "name,age,text"
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

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// A command line tool for displaying delimiter-separated data as a table in the terminal.
#[derive(Parser, Debug)]
#[command(name = "tabbs")]
#[command(
    about = "Display delimiter-separated data as a table",
    long_about = "Reads delimiter-separated values from stdin or a file and prints them as a formatted table. \
                  Supports custom delimiters (comma, tab, pipe, etc.) and optional header/cell colors."
)]
struct Args {
    /// Column names, separated by the delimiter (e.g., "name,age,text"). Not needed with --header-row.
    #[arg(short = 'c', long)]
    columns: Option<String>,

    /// Read input from file instead of stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    /// Field delimiter (default: comma)
    #[arg(short = 'd', long, default_value = ",")]
    delimiter: String,

    /// Use the first line of input as column names
    #[arg(long)]
    header_row: bool,

    /// Do not print the header row
    #[arg(long)]
    no_header: bool,

    /// Right-align columns that contain only numbers
    #[arg(long)]
    align_numeric: bool,

    /// Color for the header row (e.g., blue, red, green)
    #[arg(long)]
    header_color: Option<String>,

    /// Color for the cell text (e.g., blue, red, green)
    #[arg(long)]
    cell_color: Option<String>,
}

fn main() {
    let args = Args::parse();

    let delimiter: &str = args.delimiter.as_str();
    if delimiter.is_empty() {
        eprintln!("Error: delimiter cannot be empty");
        std::process::exit(1);
    }

    if args.columns.is_none() && !args.header_row {
        eprintln!("Error: either --columns (-c) or --header-row is required");
        std::process::exit(1);
    }

    let mut input = String::new();
    if let Some(path) = &args.file {
        input = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error: failed to read file {}: {e}", path.display());
            std::process::exit(1);
        });
    } else if let Err(e) = io::stdin().lock().read_to_string(&mut input) {
        eprintln!("Error: failed to read input: {e}");
        std::process::exit(1);
    }

    let (column_names, padded_rows) = if args.header_row {
        let all_rows = parse_input(&input, delimiter);
        if all_rows.is_empty() {
            eprintln!("Error: no data to parse (empty input or no valid lines)");
            std::process::exit(1);
        }
        let mut rows = all_rows;
        let header = rows.remove(0);
        let num_columns = header.len();
        let column_names: Vec<String> = header;
        let padded_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| {
                let mut padded = row;
                padded.resize(num_columns, String::new());
                padded
            })
            .collect();
        (column_names, padded_rows)
    } else {
        let column_names: Vec<String> = args
            .columns
            .as_deref()
            .unwrap()
            .split(delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if column_names.is_empty() || column_names.iter().all(|s| s.is_empty()) {
            eprintln!("Error: at least one non-empty column name is required");
            std::process::exit(1);
        }

        let rows = parse_input(&input, delimiter);
        let num_columns = column_names.len();
        let padded_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| {
                let mut padded = row;
                padded.resize(num_columns, String::new());
                padded
            })
            .collect();
        (column_names, padded_rows)
    };

    let column_names_ref: Vec<&str> = column_names.iter().map(|s| s.as_str()).collect();
    let mut stdout = io::stdout().lock();
    if let Err(e) = print_table_to_writer(
        &column_names_ref,
        &padded_rows,
        args.header_color.as_deref(),
        args.cell_color.as_deref(),
        args.no_header,
        args.align_numeric,
        &mut stdout,
    ) {
        eprintln!("Error: failed to write output: {e}");
        std::process::exit(1);
    }
}

/// Parse delimiter-separated input into rows of cells.
///
/// Skips empty lines and trims whitespace from each cell.
///
/// # Arguments
///
/// * `input` - The raw input string (typically from stdin).
/// * `delimiter` - The field delimiter (e.g. `,`, `\t`, `|`).
///
/// # Returns
///
/// A vector of rows, where each row is a vector of cell strings.
fn parse_input(input: &str, delimiter: &str) -> Vec<Vec<String>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split(delimiter)
                .map(|s| s.trim().to_string())
                .collect()
        })
        .collect()
}

fn is_numeric(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    s.parse::<f64>().is_ok()
}

/// Determine which columns should be right-aligned (all data cells are numeric).
fn numeric_columns(column_names: &[&str], rows: &[Vec<String>]) -> Vec<bool> {
    let mut numeric = vec![true; column_names.len()];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < numeric.len() && numeric[i] && !is_numeric(cell) {
                numeric[i] = false;
            }
        }
    }
    numeric
}

/// Print a table with the given column names, row data, optional header and cell colors to the provided writer.
///
/// # Arguments
///
/// * `column_names` - A slice of strings representing the column names.
/// * `rows` - A slice of Vec<String> representing the rows of data (each row should have the same length as column_names).
/// * `header_color` - An optional string specifying the color of the header text.
/// * `cell_color` - An optional string specifying the color of the cell text.
/// * `no_header` - If true, do not print the header row.
/// * `align_numeric` - If true, right-align columns that contain only numbers.
/// * `writer` - A mutable reference to a writer implementing the `Write` trait.
///
/// # Returns
///
/// `Ok(())` on success, or an `Err` if writing fails.
fn print_table_to_writer(
    column_names: &[&str],
    rows: &[Vec<String>],
    header_color: Option<&str>,
    cell_color: Option<&str>,
    no_header: bool,
    align_numeric: bool,
    writer: &mut impl Write,
) -> io::Result<()> {
    let mut column_widths: Vec<usize> = column_names.iter().map(|s| s.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                column_widths[i] = column_widths[i].max(cell.len());
            }
        }
    }

    let right_align = if align_numeric {
        numeric_columns(column_names, rows)
    } else {
        vec![false; column_names.len()]
    };

    let separator: String = column_widths
        .iter()
        .map(|width| "-".repeat(width + 2))
        .collect::<Vec<String>>()
        .join("+");

    writeln!(writer, "+{}+", separator)?;

    if !no_header {
        write!(writer, "|")?;
        for (i, column_name) in column_names.iter().enumerate() {
            let padded = if right_align.get(i) == Some(&true) {
                format!("{:>width$}", column_name, width = column_widths[i])
            } else {
                format!("{:<width$}", column_name, width = column_widths[i])
            };
            let display = match header_color {
                Some(color) => padded.color(color).to_string(),
                None => padded,
            };
            write!(writer, " {} |", display)?;
        }
        writeln!(writer)?;
        writeln!(writer, "+{}+", separator)?;
    }

    for row in rows {
        write!(writer, "|")?;
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                let padded = if right_align.get(i) == Some(&true) {
                    format!("{:>width$}", cell.as_str(), width = column_widths[i])
                } else {
                    format!("{:<width$}", cell.as_str(), width = column_widths[i])
                };
                let display = match cell_color {
                    Some(color) => padded.color(color).to_string(),
                    None => padded,
                };
                write!(writer, " {} |", display)?;
            }
        }
        writeln!(writer)?;
    }

    writeln!(writer, "+{}+", separator)?;
    Ok(())
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
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                None,
                None,
                false,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }

    #[test]
    fn test_print_table_with_varying_column_counts() {
        let column_names = ["x", "y", "z"];
        let rows = [
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string(), "e".to_string()],
            vec!["f".to_string()],
        ];

        let expected_output = "\
+---+---+---+
| x | y | z |
+---+---+---+
| a | b |   |
| c | d | e |
| f |   |   |
+---+---+---+";
        let mut output = Vec::new();

        let padded_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                let mut padded = row.clone();
                padded.resize(column_names.len(), String::new());
                padded
            })
            .collect();

        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &padded_rows,
                None,
                None,
                false,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }

    #[test]
    fn test_print_table_empty_input() {
        let column_names = ["a", "b"];
        let rows: Vec<Vec<String>> = vec![];

        let expected_output = "\
+---+---+
| a | b |
+---+---+
+---+---+";
        let mut output = Vec::new();

        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                None,
                None,
                false,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }

    #[test]
    fn test_print_table_with_colors() {
        let column_names = ["name", "age"];
        let rows = [
            vec!["alice".to_string(), "30".to_string()],
            vec!["bob".to_string(), "25".to_string()],
        ];

        let mut output = Vec::new();
        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                Some("blue"),
                Some("green"),
                false,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().lines().collect();
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0], lines[2]);
        assert_eq!(lines[0], lines[5]);
        assert!(lines[1].contains("name"));
        assert!(lines[1].contains("age"));
    }

    #[test]
    fn test_parse_input_empty() {
        let rows = parse_input("", ",");
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_input() {
        let input = "a,b,c\nd,e,f";
        let rows = parse_input(input, ",");
        assert_eq!(
            rows,
            [
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
                vec!["d".to_string(), "e".to_string(), "f".to_string()],
            ]
        );
    }

    #[test]
    fn test_parse_input_filters_empty_lines() {
        let input = "a,b\n\n\nc,d\n";
        let rows = parse_input(input, ",");
        assert_eq!(
            rows,
            [
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string()],
            ]
        );
    }

    #[test]
    fn test_parse_input_trims_cells() {
        let input = "  a  ,  b  \n  c  ,  d  ";
        let rows = parse_input(input, ",");
        assert_eq!(
            rows,
            [
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string()],
            ]
        );
    }

    #[test]
    fn test_parse_input_custom_delimiter() {
        let input = "a|b|c\nd|e|f";
        let rows = parse_input(input, "|");
        assert_eq!(
            rows,
            [
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
                vec!["d".to_string(), "e".to_string(), "f".to_string()],
            ]
        );
    }

    #[test]
    fn test_print_table_no_header() {
        let column_names = ["name", "age"];
        let rows = [
            vec!["alice".to_string(), "30".to_string()],
            vec!["bob".to_string(), "25".to_string()],
        ];

        let expected_output = "\
+-------+-----+
| alice | 30  |
| bob   | 25  |
+-------+-----+";
        let mut output = Vec::new();

        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                None,
                None,
                true,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }

    #[test]
    fn test_print_table_align_numeric() {
        let column_names = ["name", "age", "score"];
        let rows = [
            vec!["alice".to_string(), "30".to_string(), "95.5".to_string()],
            vec!["bob".to_string(), "25".to_string(), "87".to_string()],
        ];

        let expected_output = "\
+-------+-----+-------+
| name  | age | score |
+-------+-----+-------+
| alice |  30 |  95.5 |
| bob   |  25 |    87 |
+-------+-----+-------+";
        let mut output = Vec::new();

        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                None,
                None,
                false,
                true,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }

    #[test]
    fn test_print_table_empty_column_names() {
        let column_names = ["", ""];
        let rows = [vec!["x".to_string(), "y".to_string()]];

        let expected_output = "\
+---+---+
|   |   |
+---+---+
| x | y |
+---+---+";
        let mut output = Vec::new();

        {
            let mut output_writer = std::io::BufWriter::new(&mut output);
            print_table_to_writer(
                &column_names,
                &rows,
                None,
                None,
                false,
                false,
                &mut output_writer,
            )
            .unwrap();
        }
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), expected_output);
    }
}
