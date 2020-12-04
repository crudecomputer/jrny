use chrono::{DateTime, Local, Utc};
use clap::{clap_app, AppSettings};
use log::{info, warn, LevelFilter};
use std::path::PathBuf;

use jrny::{commands, Config, Executor, Logger};

static LOGGER: Logger = Logger;

fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();

    let app = clap_app! {jrny =>
        (about: "Data modeling is a journey; manage yours with jrny - simple PostgreSQL schema management")
        (version: env!("CARGO_PKG_VERSION"))
        (setting: AppSettings::SubcommandRequired)

        (@subcommand begin =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        (@subcommand plan =>
            (about: "Generates a timestamped SQL revision")
            (@arg name: +required "Name of the revision")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
        )

        (@subcommand review =>
            (about: "Reviews which plans need to be applied, which have been applied and when, and whether or not plans already applied appear to differ from the plan file")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
        )

        (@subcommand embark =>
            (about: "Reviews and applies the available revisions")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
            (@arg commit: --commit !takes_value "Commits the transaction, false by default to encourage dry runs")
        )
    };

    let result = match app.clone().get_matches().subcommand() {
        ("begin", Some(cmd)) => begin(cmd.value_of("dirpath").unwrap()),
        ("plan", Some(cmd)) => plan(cmd.value_of("name").unwrap(), cmd.value_of("config")),
        ("review", Some(cmd)) => review(cmd.value_of("config")),
        ("embark", Some(cmd)) => embark(cmd.value_of("config"), cmd.is_present("commit")),
        _ => unreachable!(),
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
}

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(path: &str) -> Result<(), String> {
    let cmd = commands::Begin::new_project(path)?;

    info!("A journey has begun");

    print_path("  ", cmd.created_root, &cmd.paths.root);
    print_path("  ├── ", cmd.created_revisions, &cmd.paths.revisions);
    print_path("  └── ", cmd.created_conf, &cmd.paths.conf);

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn plan(name: &str, conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let cmd = commands::Plan::new_revision(&config, name)?;

    info!("Created {}", cmd.filename);

    Ok(())
}

pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    let cmd = commands::Review::annotated_revisions(&config, &mut exec)?;

    if cmd.revisions.is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");

        return Ok(());
    }

    info!("The journey thus far\n");
    info!("{:50}{:25}{:25}", "Revision", "Created", "Applied");

    let format_local = |dt: DateTime<Utc>| DateTime::<Local>::from(dt).format("%v %X").to_string();

    for revision in cmd.revisions {
        let applied_on = match revision.applied_on {
            Some(a) => format_local(a),
            _ => "--".to_string(),
        };

        let error = if let Some(false) = revision.checksums_match {
            Some("The file has changed after being applied")
        } else if !revision.on_disk {
            Some("No corresponding file could not be found")
        } else {
            None
        };

        match error {
            Some(error) => warn!(
                "{:50}{:25}{:25}{}",
                revision.filename,
                format_local(revision.created_at),
                applied_on,
                error,
            ),
            None => info!(
                "{:50}{:25}{:25}",
                revision.filename,
                format_local(revision.created_at),
                applied_on,
            ),
        }
    }

    Ok(())
}

pub fn embark(conf_path_name: Option<&str>, commit: bool) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    let cmd = commands::Embark::prepare(&config, &mut exec)?;

    if cmd.to_apply.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    info!("Found {} revision(s) to apply", cmd.to_apply.len());

    for revision in &cmd.to_apply {
        info!("\t{}", revision.filename);
    }

    cmd.apply(&mut exec, commit)?;

    if commit {
        println!("\nCommitting the transaction")
    } else {
        println!("\nRolling back the transaction - use `--commit` to persist changes")
    }

    Ok(())
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, created: bool, path: &PathBuf) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
