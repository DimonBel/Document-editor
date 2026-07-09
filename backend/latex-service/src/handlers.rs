// The actual LaTeX parser, OMML emitter, and DOCX writer were moved out of the legacy
// `backend/src/latex/` tree and re-shaped into axum handlers. The legacy module is now
// archived under `legacy/backend/src/latex/` and the new module is structured as:
//
//   parser.rs      -- LaTeX -> AST
//   omml.rs        -- AST -> OMML XML
//   docx_writer.rs -- AST -> DOCX zip
//   http.rs        -- axum handlers (re-exported as `compile` and `to_docx`)
//
// For the initial cut the handlers return a synchronous "queued" stub; the full
// port retains the MAX_SOURCE_BYTES = 1 MiB guard and the `-no-shell-escape` rule.
pub const MAX_SOURCE_BYTES: usize = 1_048_576;
