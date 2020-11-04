use crate::Executor;

// connect to database
//
// check plan table is created
//     if not, create table
//
// gather plan file information
//     base filename & timestamp separated
//     timestamp file last updated
//     checksum
pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let mut exec = Executor::new(conf_path_name)?;

    exec.ensure_table_exists()?;

    Ok(())
}
