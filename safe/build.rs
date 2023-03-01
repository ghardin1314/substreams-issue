use anyhow::{Ok, Result, Error};
use common::codegen;

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rerun-if-changed=abi");
    codegen::generate(None)?;
    Ok(())
}
