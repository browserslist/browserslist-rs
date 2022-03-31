# browserslist-rs

The tool like [Browserslist](https://github.com/browserslist/browserslist), but written in Rust.

## Project Status

> Can I use this library?

If you don't rely on the features mentioned in the [Limitations](#limitations) section,
you can use it.

We have supported most widely or most frequently used queries,
and there are over 100 tests to make sure it works correctly.

For more detail about development status, please see [Project #1](https://github.com/g-plane/browserslist-rs/projects/1).

## Usage

### Using as an NPM package

Install it:

```
npm i -D browserslist-rs
```

Since the **main** API is same as [JavaScript-based Browserslist](https://github.com/browserslist/browserslist),
you can just do as before:

```js
const browserslist = require('browserslist-rs')

browserslist('last 2 versions, not dead')
```

Note that we don't plan to provide full API compatibility,
so only the main exported API is available.

### Using as a Rust crate

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

- Custom usage like `> 0.5% in my stats`.
- Custom usage like `cover 99.5% in my stats`.
- Extending custom config like `extends browserslist-config-mycompany`.

## Credits

Thanks [Andrey Sitnik](https://github.com/ai) for creating the [JavaScript-based Browserslist](https://github.com/browserslist/browserslist) which is under MIT License.

## License

MIT License

Copyright (c) 2021-present Pig Fang
