# Contributing

Welcome stranger!

If you have come here to learn how to contribute to mdBook, we have some tips for you!

First of all, don't hesitate to ask questions!
Use the [issue tracker](https://github.com/rust-lang/mdBook/issues), no question is too simple.

## Code Quality

We love code quality and Rust has some excellent tools to assist you with contributions.

### Formatting Code with rustfmt

Before you make your Pull Request to the project, please run it through the `rustfmt` utility.
This will ensure we have good quality source code that is better for us all to maintain.

[rustfmt](https://github.com/rust-lang/rustfmt) has a lot more information on the project.
The quick guide is

1. Install it (`rustfmt` is usually installed by default via [rustup](https://rustup.rs/)):

    ```bash
    rustup component add rustfmt
    ```

2. You can now run `rustfmt` on a single file simply by...

    ```bash
    rustfmt src/path/to/your/file.rs
    ```

    ... or you can format the entire project with

    ```bash
    cargo fmt
    ```

When run through `cargo` it will format all bin and lib files in the current package.

For more information, such as running it from your favourite editor, please see the `rustfmt` project. [rustfmt](https://github.com/rust-lang/rustfmt)

#### Finding Issues with Clippy

[Clippy](https://doc.rust-lang.org/clippy/) is a code analyser/linter detecting mistakes, and therefore helps to improve your code.
Like formatting your code with `rustfmt`, running clippy regularly and before your Pull Request will help us maintain awesome code.

1. To install

    ```bash
    rustup component add clippy
    ```

2. Running clippy

    ```bash
    cargo clippy
    ```

### Making a pull-request

When you feel comfortable that your changes could be integrated, you can create a pull-request on GitHub.
One of the core maintainers will then approve the changes or request some changes before it gets merged.

That's it, happy contributions! :tada: :tada: :tada:
