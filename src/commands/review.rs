use crate::Jrny;

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
    let jrny = Jrny::from_config(conf_path_name)?;

    Ok(())
}
