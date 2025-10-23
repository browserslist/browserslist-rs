# browserslist-rs

The tool like [Browserslist](https://github.com/browserslist/browserslist), but written in Rust.

## Project Status

> Can I use this library?

If you don't rely on the features mentioned in the [Limitations](#limitations) section,
you can use it.

We have supported most widely or most frequently used queries,
and there are over 100 tests to make sure it works correctly.

For more detail about development status, please see [Project #1](https://github.com/browserslist/browserslist-rs/projects/1).

## Usage

Please refer to [crate documentation](https://docs.rs/browserslist-rs/).

## Try as Rust crate example

You can try and inspect query result by running example with Cargo:

```sh
cargo run --example inspect -- <query>
```

You can also specify additional options, for example:

```sh
cargo run --example inspect -- --mobile-to-desktop 'last 2 versions, not dead'
```

To get more help, you can run:

```sh
cargo run --example inspect -- -h
```

## Limitations

The features below aren't supported currently:

-   Custom usage like `> 0.5% in my stats`.
-   Custom usage like `cover 99.5% in my stats`.
-   Baseline queries added in [browserslist v4.26.0](https://github.com/browserslist/browserslist/releases/tag/4.26.0) like `baseline widely available`.

## Local development setup

1. Clone the repository and enter the project directory

   ```sh
   git clone https://github.com/browserslist/browserslist-rs.git
   cd browserslist-rs
   ```

2. Initialize Git submodules

    ```sh
    git submodule update --init --recursive
    ```

3. Generate data

   ```sh
   cargo run --manifest-path generate-data/Cargo.toml
   ```

4. Run the main project (see the [Usage](#usage) section above)

## Credits

Thanks [Andrey Sitnik](https://github.com/ai) for creating the [JavaScript-based Browserslist](https://github.com/browserslist/browserslist) which is under MIT License.

## License

MIT License

Copyright (c) 2021-present Pig Fang
