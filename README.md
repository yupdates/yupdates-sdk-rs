## Yupdates Rust SDK

The Yupdates Rust SDK lets you easily use the Yupdates API from your own software and scripts.

Also see the [Yupdates Python SDK](https://github.com/yupdates/yupdates-sdk-py).

To learn how to use the HTTP+JSON API directly, see [the API documentation](https://www.yupdates.com/api/).

### Getting started

First, obtain an API token from the application. Navigate to "Settings" and then "API".

The examples will start with read-only operations, so you can use the general, read-only token to get started.

Create a new Rust project:

```shell
$ cargo new yupdates-sdk-example
```

Add the dependency to `Cargo.toml`:
```toml
[dependencies]
yupdates = "0.1"
```

Add this to the `src/main.rs` file:
```rust
TODO
```

Set one environment variable:
```shell
export YUPDATES_API_TOKEN="1a7814fc25:c38bb526..."
```

Test the connection and authentication:
```shell
cargo run
```

If there is anything but a 200 response, it will be an `Err`. Otherwise, this will print out the JSON response which is returned from the `ping` function deserialized into a Python dict.

The `ping` operation is helpful to run in the beginning of your scripts to make sure there are no setup issues.

### Examples

TODO

### Getting help

You can create a [GitHub issue](https://github.com/yupdates/yupdates-sdk-rs/issues) on this repository for bugs and feature requests.

The fastest way to get help is to create a support ticket from the Yupdates application. Or email `support@yupdates.com`. Especially you need help that is not specific to this SDK.

### License

The SDK is distributed under the MIT license, please see [LICENSE](https://github.com/yupdates/yupdates-sdk-rs/blob/main/LICENSE) for more information.

This covers the any source code and examples, but it does not cover related items like the Yupdates logo or API documentation which is not hosted here.
