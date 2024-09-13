# mdBook-Header-Footer

This mdBook preprocessor prepends headers and appends footers to
all chapters in the book whose URL path match the corresponding regex.
The headers and footers are two lists of object with the `regex` and `padding`
string fields, where `regex` specifies what Regex the URL path needs to
match and `padding` specifies what to pad.
`regex` is optional, if not specified, it will match all paths (`.*`).

For example, if you add the following to your `book.toml`:

```toml
[preprocessor.header-footer]
headers = [{ regex = "^notes/", padding = "Notes\n" }]
footers = [{ padding = "\nHaha" }]
```

Then, all chapters whose URL path starts with `notes/` will have `Notes`
plus a newline prepended to the top of the chapter;
all chapters will have a newline plus `Haha` appended to the bottom of
the chapter.

## Installation

```sh
cargo install mdbook_header_footer
```

## Debugging

We use [`tracing-subscriber` with the `env-filter`
feature](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/#feature-flags)
to emit logs.
Please configure the log level by setting the `RUST_LOG` environment variable.
