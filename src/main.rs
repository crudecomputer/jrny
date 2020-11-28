use clap::{clap_app, AppSettings};

use chrono::{DateTime, Local, Utc};
use std::{
    convert::TryFrom,
    fmt::Display,
    fs,
    io::prelude::*,
};

use jrny::{Config, Executor, RevisionFile, StatementGroup};
use jrny::commands;

fn main() {
    let app = clap_app! {jrny =>
        (about: "Data's a journey, so manage yours with jrny - simple PostgreSQL schema management")
        (version: "1.0.0")
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
        ("begin", Some(cmd)) => begin(
            cmd.value_of("dirpath").unwrap(),
        ),
        ("plan", Some(cmd)) => plan(
            cmd.value_of("name").unwrap(),
            cmd.value_of("config"),
        ),
        ("review", Some(cmd)) => review(
            cmd.value_of("config"),
        ),
        ("embark", Some(cmd)) => embark(
            cmd.value_of("config"),
            cmd.is_present("commit"),
        ),
        _ => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }
}

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(p: &str) -> Result<(), String> {
    let cmmd = commands::Begin::new_project(p)?
        .create_root()?
        .create_revisions()?
        .create_conf()?;

    println!("A journey has begun");

    print_path("  ",     cmmd.created_root,      &cmmd.paths.root.display());
    print_path("  ├── ", cmmd.created_revisions, &cmmd.paths.revisions.display());
    print_path("  └── ", cmmd.created_conf,      &cmmd.paths.conf.display());

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn plan(name: &str, conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;

    let utc: DateTime<Utc> = Utc::now();
    let timestamp = utc.timestamp();

    let revision_path = config.paths.revisions
        .join(format!("{}.{}.sql", timestamp, name));
    let filename = revision_path.display();

    fs::File::create(&revision_path)
        .map_err(|e| e.to_string())?
        .write_all(format!(
            "-- Journey revision\n--\n-- {}\n--\n\n",
            filename,
        ).as_bytes())
        .map_err(|e| e.to_string())?;

    println!("Created {}", filename);

    Ok(())
}

pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    exec.ensure_table_exists()?;

    let files = RevisionFile::all_from_disk(&config.paths.revisions)?;
    let records = exec.load_revisions()?;
    let annotated = commands::Review::annotate(files, records);

    if annotated.len() == 0 {
        println!("No revisions found. Create your first revision with `jrny revise <some-revision-name>`.");

        return Ok(());
    }

    println!("The journey thus far\n");
    println!("{:50}{:25}{:25}", "Revision", "Created", "Applied");

    let format_local = |dt: DateTime<Utc>| DateTime::<Local>::from(dt)
        .format("%v %X")
        .to_string();

    for anno in annotated {
        let applied_on = match anno.applied_on {
            Some(a) => format_local(a),
            _ => "--".to_string(),
        };

        let error = if let Some(false) = anno.checksums_match {
            "The file has changed after being applied"
        } else if !anno.on_disk {
            "No corresponding file could not be found"
        } else {
            ""
        };

        println!(
            "{:50}{:25}{:25}{}",
            anno.filename,
            format_local(anno.created_at),
            applied_on,
            error,
        );
    }

    Ok(())
}

pub fn embark(conf_path_name: Option<&str>, commit: bool) -> Result<(), String> {
    // FIXME this is SO much copy/paste with `review`
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    // Review revisions
    let files = RevisionFile::all_from_disk(&config.paths.revisions)?;
    let records = exec.load_revisions()?;
    let annotated = commands::Review::annotate(files, records);

    // If checksum comparison is missing, it hasn't been applied so ignore it
    let changed: Vec<_> = annotated.iter()
        .filter(|anno| !anno.checksums_match.unwrap_or(true))
        .collect();

    let missing: Vec<_> = annotated.iter()
        .filter(|anno| !anno.on_disk)
        .collect();

    if changed.len() > 0 || missing.len() > 0 {
        let mut msg = "Failed to run revisions".to_string();

        if changed.len() > 0 {
            msg.push_str(&format!("{} have changed since being applied", changed.len()));
        }

        if missing.len() > 0 {
            msg.push_str(&format!("{} are no longer present on disk", changed.len()));
        }

        return Err(msg);
    }

    let to_apply: Vec<_> = annotated.iter()
        .filter(|anno|
            anno.on_disk &&
            anno.applied_on.is_none()
        )
        .collect();

    if to_apply.len() == 0 {
        println!("No revisions to apply");
        return Ok(())
    }

    println!("Found {} revision(s) to apply", to_apply.len());

    for revision in &to_apply {
        println!("\t{}", revision.filename);
    }

    // TODO confirm..? or allow "auto confirm" option..?
    // Parse all files into statements before printing or applying any
    let mut groups = vec![];

    for revision in to_apply {
        match StatementGroup::try_from(revision.contents.as_ref().unwrap().as_str()) {
            Ok(group) => {
                groups.push((revision, group));
            },
            Err(e) => {
                eprintln!("\nFound error in \"{}\"", revision.filename);
                return Err(e);
            },
        }
    }

    let _ = exec.run_revisions(groups, commit)?;

    Ok(())
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, created: bool, path_name: impl Display) {
    println!(
        "{}{}{}",
        prefix,
        path_name,
        if created { " [created]" } else { "" },
    );
}
