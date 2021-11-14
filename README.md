# browserslist-rs

The tool like [Browserslist](https://github.com/browserslist/browserslist), but written in Rust.

## Try it out

Before trying this crate, you're required to get [Rust](https://www.rust-lang.org/) installed.
Then, clone this repository.

Now, you can try and inspect query result by running example with Cargo:

```sh
cargo run --example inspect -- <query>
```

You can also specify additional options, for example:

```sh
cargo run --example inspect -- --mobile-to-desktop 'last 2 versions, not dead`
```

To get more help, you can run:

```sh
cargo run --example inspect -- -h
```

## Limitations

The features below aren't supported currently:

- Custom usage like `> 0.5% in my stats` or `> 0.5% in XX`.
- Custom usage like `cover 99.5% in my stats` or `cover 99.5% in XX`.
- Extending custom config like `extends browserslist-config-mycompany`.
- Specifying feature name like `supports es6-module`.
- The `browserslist config` query.

## License

MIT License

Copyright (c) 2021-present Pig Fang
