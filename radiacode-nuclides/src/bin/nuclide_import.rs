mod import;

use import::cli::parse_import_options;
use import::output::ImportResult;
use import::pipeline::run_import;

fn main() -> ImportResult<()> {
    run_import(parse_import_options())
}
