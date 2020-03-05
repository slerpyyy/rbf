# rbf
*A simple optimizing Brainfuck interpreter written in Rust*

rbf (short for either "Run BrainFuck" or "Rust BrainFuck") is a simple program you can use to run brainfuck code straight from your command line. Code can be read from a file, stdin or directly from a command-line argument. The input for the brainfuck program is stdin by default but can be specified using command-line arguments.

## Internal Representation

To speed up evaluation, rbf uses an internal representation which is generated from the brainfuck code. The IR mainly consists of the following instructions:

| Instruction | Example | Description |
|---|---|---|
| `Touch(high, low)` | | Makes sure cells in range ]low+index, high+index] are allocated. |
| `Set(off, val)` | `[-]+++` | Set cell at index+off to val. |
| `Add(off, val)` | `+++++` | Add val to the cell at index+off. |
| `Mul(off, val)` | `[>++<-]` | Multiply val with the register and add the result to the cell at index+off. |
| `Move(off)` | `>>>>` | Add off to the current index. |
| `Store(off)` | | Write the value of the cell at index+off to the register. |
| `Loop(sub)` | `[>>+++]` | Run the sub program while the cell at index is not 0. |
| `Scan(val, step)` | `[->>+]` | Begin at index and move by step until a cell with a value of val is found. |
| `Fill(off, val, step)` | `[[-]++>]` | Begin at index and move by step while setting cells with<br>offset off to val until a cell with value 0 is found. |
| `Input(off)` | `,` | Receive one byte and write it to the cell at index+off. |
| `Output(off)` | `.` | Send the value of the cell at index+off. |

You can take a look at the generated IR using the `--static` flag.

## Setup

This program requires rust to be installed. You can find a guide on how to install rust on the [official website](https://www.rust-lang.org/learn/get-started). After that, everything else should happen automatically:

```
git clone https://github.com/slerpyyy/rbf.git
cd rbf
cargo install --path .
```

## Usage

You can get a quick guide on how to run rbf by running it with the `--help` flag or without any command-line arguments:

```
Usage: rbf FILE [options]

A simple optimizing Brainfuck interpreter written in Rust.

Options:
    -h, --help          print this help message
    -c, --cmd CODE      run code from command
    -i, --input TEXT    use string as input
    -f, --force         allow invalid code to run
    -s, --static        only print the generated code
```

## Extra Credits

Some basic ideas of this project go back to [this post](http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html) by Mats Linander titled "brainfuck optimization strategies".
