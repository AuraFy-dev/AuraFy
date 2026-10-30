//! Local wrapper exposing the uniffi_bindgen Kotlin backend as a binary:
//! `cargo run --bin uniffi-bindgen -- <cdylib> <out-dir>`.
use camino::Utf8PathBuf;
use uniffi_bindgen::bindings::KotlinBindingGenerator;
use uniffi_bindgen::EmptyCrateConfigSupplier;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let library = Utf8PathBuf::from(args.next().expect("usage: uniffi-bindgen <cdylib> <out-dir>"));
    let out_dir = Utf8PathBuf::from(args.next().expect("usage: uniffi-bindgen <cdylib> <out-dir>"));
    uniffi_bindgen::library_mode::generate_bindings(
        &library,
        None,
        &KotlinBindingGenerator,
        &EmptyCrateConfigSupplier,
        None,
        &out_dir,
        false,
    )?;
    Ok(())
}
