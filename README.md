# doxy-to-md

[Material for MkDocs]: https://squidfunk.github.io/mkdocs-material/

Converts Doxygen XML output to Markdown files, for use with [Material for MkDocs].

## Disclaimer

[Centurion]: https://github.com/albin-johansson/centurion/

This tool was written for use by the [Centurion] project. It is not intended to perform perfect or 100% complete
conversions from Doxygen to Markdown. The goal is to provide a good overview of the APIs with minimal manual work to get
the API documentation available in a Material for MkDocs environment.

## Building

This tool is written in Rust, which makes is super easy to build the tool locally. All you need is the Rust toolchain.

Simply clone and unzip the repository to somewhere on your machine and enter the following command in a shell, opened
in the root directory of the repository.

```shell
cargo build --release
```

When the compilation is done, you should be able to locate the binary, called `doxy-to-md`, in `target/release`.

## Usage

This tool processes the XML format emitted by Doxygen. However, Doxygen only emits HTML files by default, so you'll need
to set `GENERATE_XML` to `YES` in your Doxyfile. Then, run Doxygen on your sources to produce the XML files.

To run the `doxy-to-md` program, you need to supply two arguments, `-i`/`--input-dir` and `-o`/`--output-dir`. The input
directory is where `doxy-to-md` looks for the XML files produced by Doxygen. Subsequently, the output directory is where
the generated Markdown files will go.

The output directory will be created if it does not exist by the time `doxy-to-md` is executed.

```shell
./doxy-to-md -i path/to/doxygen/xml -o output/md
```
