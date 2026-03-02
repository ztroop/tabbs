[![Build](https://github.com/ztroop/tabbs/actions/workflows/build.yml/badge.svg)](https://github.com/ztroop/tabbs/actions/workflows/build.yml)

# tabbs

A command line tool for displaying delimiter-separated data as a table in the terminal.

## Features

- Takes input from stdin or a file
- Supports specifying column names or using first row as header
- Customizable delimiter (comma, tab, pipe, etc.)
- Right-align numeric columns
- Configurable header and cell colors
- Easy to use

## Installation

To install tabbs, you will need [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

```sh
cargo install --path .
```

Or install from crates.io:

```sh
cargo install tabbs
```

## Usage

Pipe delimiter-separated data to tabbs and specify column names with the `-c` flag:

```sh
printf "jack,35,neat\njane,50,cool\nerin,20,nice" | tabbs -c "name,age,text"
```

Output:

```
+------+-----+------+
| name | age | text |
+------+-----+------+
| jack | 35  | neat |
| jane | 50  | cool |
| erin | 20  | nice |
+------+-----+------+
```

### Command line options

| Option | Short | Description |
|--------|-------|-------------|
| `--columns` | `-c` | Column names, separated by the delimiter (required unless `--header-row`) |
| `--file` | `-f` | Read input from file instead of stdin |
| `--delimiter` | `-d` | Field delimiter (default: `,`) |
| `--header-row` | | Use the first line of input as column names |
| `--no-header` | | Do not print the header row |
| `--align-numeric` | | Right-align columns that contain only numbers |
| `--header-color` | | Color for the header row |
| `--cell-color` | | Color for cell text |
| `--help` | `-h` | Print help |

Supported colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, and their `bright_` variants.

### File input

Read from a file with `-f`:

```sh
tabbs -f data.csv -c "name,age,city"
```

### Header from first row

When your data includes a header line, use `--header-row` instead of `-c`:

```sh
printf "name,age\njack,35\njane,50" | tabbs --header-row
```

### Right-align numbers

Use `--align-numeric` to right-align columns where all values are numbers:

```sh
printf "name,age,score\nalice,30,95.5\nbob,25,87" | tabbs --header-row --align-numeric
```

### Custom delimiter

Use `-d` or `--delimiter` for tab-separated, pipe-separated, or other formats:

```sh
# Tab-separated (use $'\t' in bash; column names must use the same delimiter)
printf "alice\t30\nbob\t25" | tabbs -c $'name\tage' -d $'\t'

# Pipe-separated
printf "alice|30\nbob|25" | tabbs -c "name|age" -d "|"
```

### Colors

```sh
printf "jack,35,neat\njane,50,cool\nerin,20,nice" | tabbs -c "name,age,text" --header-color blue --cell-color green
```

## Limitations

- Simple delimiter-separated parsing only; quoted fields (e.g. `"Smith, John",42`) are not supported
- Rows with fewer columns than the header are padded with empty cells
- Rows with more columns than the header are truncated
- Empty lines in input are skipped
- Input is read fully into memory (very large files may use significant memory)

## Contributing

Contributions are welcome. Please open an issue or pull request on [GitHub](https://github.com/ztroop/tabbs).

## License

MIT License. See [LICENSE](LICENSE) for details.
