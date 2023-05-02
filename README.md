# tabbs

A command line tool for displaying comma-separated data as a table in the terminal.

## Features

- Takes input from stdin
- Supports specifying column names
- Configurable header and cell colors
- Easy to use

## Installation

To install tabb, you will need Rust and Cargo installed on your system. You can find installation instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

Once Rust and Cargo are installed, you can build and install tabb using the following command:

```sh
$ cargo install --path .
```

## Usage

To use tabb, you need to pipe input data to the tool, specifying the column names with the `-c` flag:

```sh
$ echo "jack,35,neat\n\
        jane,50,cool\n\
        erin,20,nice" | tabb -c "name,age,text"
```

This will produce the following output:

```sh
+------+-----+------+
| name | age | text |
+------+-----+------+
| jack | 35  | neat |
| jane | 50  | cool |
| erin | 20  | nice |
+------+-----+------+
```

## Specifying Colors

You can also specify the header and cell colors using the `--header-color` and `--cell-color` flags:

```sh
$ echo "jack,35,neat\n\
        jane,50,cool\n\
        erin,20,ah" | tabb -c "name,age,text" --header-color blue --cell-color green
```

This will produce a table with blue headers and green cell text.

## Contributing

If you would like to contribute to the project, feel free to submit a pull request on GitHub.

## License

This project is released under the MIT License. See the LICENSE file for details.
