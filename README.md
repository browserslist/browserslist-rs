# browserslist-rs

The tool like [Browserslist](https://github.com/browserslist/browserslist), but written in Rust.

For the project status, please see [Project #1](https://github.com/g-plane/browserslist-rs/projects/1).

## Usage

### Using as an NPM package

Install it:

```
npm i -D browserslist-rs
```

Since the **main** API is same as [JavaScript-based Browserslist](https://github.com/browserslist/browserslist),
you can just do as before:

```diff
- const browserslist = require('browserslist')
+ const browserslist = require('browserslist-rs')

browserslist('last 2 versions, not dead')
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
