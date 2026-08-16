# aigov

`aigov` is a Rust library and command-line tool for inspecting and modifying OOXML Office documents. Library document
opening, metadata updates, and text replacement currently support DOCX files.

## Quick start

Build and run the command-line tool from a checkout:

```sh
cargo run -- report.docx metadata --pretty
```

## Install

Install the release build so that `aigov` is available from any directory:

```sh
cargo make install
```

The task installs to `~/.local/bin`. Add that directory to your `PATH` once:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zprofile
source ~/.zprofile
```

Run `cargo make install` again after source changes to update the installed binary.

Update metadata while preserving properties not named in the command:

```sh
cargo run -- report.docx metadata set \
  --title "Quarterly report" \
  --custom Department=Finance
```

Replace text in a DOCX document, writing the result to a new file:

```sh
cargo run -- report.docx replace \
  --search Draft \
  --replace Final \
  --output published-report.docx
```

Use `cargo run -- <FILEPATH> --help` for all available commands. PDF conversion requires LibreOffice and the `soffice`
command on `PATH`.

## Library

Open a document through format dispatch, reject structural errors, and save a copy:

```rust,no_run
use aigov::{
    document::Severity,
    formats::Document,
    office::OfficeDocument,
};
use std::path::Path;

let document = Document::from_file(Path::new("report.docx"))?;

if document
    .validate()?
    .iter()
    .any(|issue| issue.severity == Severity::Error)
{
    return Err("refusing to publish an invalid document".into());
}

document.save(Path::new("published-report.docx"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For concrete DOCX construction and OOXML package access, see
[`DocxDocument`](https://docs.rs/aigov/latest/aigov/formats/struct.DocxDocument.html),
[`OfficeDocument`](https://docs.rs/aigov/latest/aigov/office/trait.OfficeDocument.html), and [
`OoxmlDocument`](https://docs.rs/aigov/latest/aigov/office/trait.OoxmlDocument.html).

## Development

```sh
cargo test
cargo test --doc
cargo doc --no-deps
```

## Publishing

Cargo includes this README on the package page because `crates/aigov/Cargo.toml` declares
`readme = "README.md"`. Before publishing publicly, set the package `license`
and `repository` fields (and `homepage`, `keywords`, and `categories` when they are known).
