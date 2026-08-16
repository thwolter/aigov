# Architecture

## A01: DOCX policy codec owns XML mechanics

- **Statement**: DOCX policy codec functions accept raw document XML and internally construct `quick_xml` readers and
  writers; `DocxDocument` owns only package-part reads and writes.
- **Status**: active
- **Provenance**: user-revised
- **Evidence**: [N05, N06]
- **Code refs**: [`../../../crates/aigov/src/formats/docx/policy.rs`, `../../../crates/aigov/src/formats/docx.rs`]
- **Last revised**: 2026-08-11 (2026-08-11_001#5)
