// Format specification display
use crate::cli::output::format_specs::display_scid_format_specifications;

pub fn execute() -> std::io::Result<()> {
    display_scid_format_specifications();
    Ok(())
}

