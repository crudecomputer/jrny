# Journey

*Data modeling is a journey - manage yours with `jrny`*

**Important:** Journey is still very much a prototype; being version >= 1 simply means that it reached minimum required (and working) features, and development continues to be sporadic due to the responsibilities of life.

In other words: **USE WITH DISCRETION**

## Overview

Other SQL-based schema migration tools already exist (like [dbmate](<https://github.com/amacneil/dbmate>)),
but there is still room for another.

`jrny` is an option for people who...

* ... think database revision files should be an immutable record and are **guaranteed to represent** what was applied to database

* ... want a **guaranteed revision order** across all environments

* ... would **rather write SQL** than translate it to method calls or YAML entries that are often more verbose and less documented

* ... want **explicit control of transactions** and the ability to easily ignore them or leverage them across multiple revisions

* ... prefer to **install compiled binaries** rather than manage a language and dependencies on whatever system(s) run migrations

* ... like the idea of **single responsibility**, especially if multiple applications (potentially in different repos and written in different languages) access the same tables

* ... believe that **separating migrations from application deploys** encourages one to write non-breaking migrations and helps enable zero-downtime updates

* ... prefer to **avoid reverse migrations**, especially as they make it trivially easy to 'change' history by forgetting to add a preexisting index, check constraint, etc. during a typical upgrade/downgrade/edit/upgrade cycle. (Feel free to add your thoughts on this subject [here](<https://github.com/kevlarr/jrny/issues/25>))

## CLI Usage

`jrny` is primarily intended to be used as a precompiled, standalone CLI tool,
but it can also be used [as a library](#library-usage).

### Installation

#### From source

Assuming `cargo` is installed (easiest is using [rustup](<https://rustup.rs/>)) then simply run:

```bash
$ cargo install jrny --version 2.0.0-beta.7

    Updating crates.io index
  Downloaded jrny v2.0.0-beta.7
  Downloaded 1 crate (28.6 KB) in 0.39s
  Installing jrny v2.0.0-beta.7
   ...
   ...
   ...
   Compiling jrny v2.0.0-beta.7
    Finished release [optimized] target(s) in 2m 03s
  Installing /Users/<user>/.cargo/bin/jrny
   Installed package `jrny v2.0.0-beta.7` (executable `jrny`)
```

### Usage

There are **4 steps** to managing schema changes with `jrny`:

1. `begin`
2. `plan`
3. `review`
4. `embark`

#### Begin the journey

Project setup is simple - all that is required is a config file and an empty revisions directory alongside it.
These can be created manually or via `jrny begin`.

```bash
$ jrny begin <project-dir>

A journey has begun
  <project-dir>
  ├── <project-dir>/revisions [created]
  └── <project-dir>/jrny.toml [created]
  └── <project-dir>/jrny-env.toml [created]
  └── <project-dir>/jrny-env.example.toml [created]
  
```

The default `jrny.toml` file specifies the directory in which to
locate revisions as well as the database schema & name for the
"state table" in which to record details for applied revisions.

```toml
# jrny.toml

[revisions]
directory = "revisions"

[table]
schema = "public"
name = "jrny_revision"
```

The revision directory can be renamed any time, provided that the SQL
files themselves do not change,
but the schena & table cannot be changed once any revisions have been applied.
Otherwise, `jrny` will see an empty state table and attempt to
apply all revisions again.

Additionally, `jrny-env.toml` and `jrny-env.example.toml` files will be created.
The `jrny-env.toml` environment file is optional but is used to store
environment-specific information, including the database connection string.

```toml
# jrny-env.example.toml

[database]
# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = "postgresql://user:password@host:port/dbname"
```

Both the config and environment files can be freely renamed,
but changing their names (or running `jrny` outside of the
project directory) will require passing in their paths via
`-c [or --config]` and `-e [or --environment]` respectively.

#### Plan the journey

To create a new SQL revision, run `jrny plan [-c <path-to-config>]` either specifying the path to the
config file via `-c` or (if ommitted) by looking for `jrny.toml` in the current directory.

```bash
$ jrny plan create-users

Created revisions/001.1606743300.create-users.sql

$ jrny plan 'name with spaces' -c /path/to/my/config.toml

Created /path/to/my/revisions/002.1606743400.name with spaces.sql
```

This will create a (mostly) empty SQL file for you to populate with wonderful statements.
Notice that `jrny` **encourages transactions per-revision** but you are free to remove these,
particularly if you need to execute statements outside of a transaction - or if you want to write several revision files that should
span the same transaction.

``` bash
$ cat /path/to/my/revisions/002.1606743400.name\ with\ spaces.sql

-- Revision: name with spaces
--
-- Add description here

begin;

-- Add SQL here

commit;
```

> Note: It's encouraged to comment-out the `commit;` line so that you
> can run the revision in the database without changes actually persisting.

Revision filenames follow the pattern of `[id].[timestamp].[name].sql`.

Timestamps are just great metadata to capture, and `jrny` assigns a sequential id to each file.
The reason being this enforces a stricter revision order than simply using timestamps can,
all without needing pointers between files.
(For more information, see the [rational behind sequencing](<https://github.com/kevlarr/jrny/issues/17>).)

Gaps in the id sequence are fine (eg. if you create two new revisions, remove the first one, and then apply the second),
and ids can be manually changed as long as the revision hasn't been applied.

#### Review the journey

To summarize the state of revisions, run `jrny review`.
If you are outside of the project directory, you'll need to
specify the config file location, and
you will either need to specify the path to the environment
file or provide the database URL directly, eg:

```bash
# From within project directory & default filenames
$ jrny review

# From outside the project directory *or* with a custom config filename.
#
# This will look for an environment file named `jrny-env.toml` in
# the same directory as the custom config file.
$ jrny review -c path/to/my-jrny-config.toml

# Same as above except can specify custom environment file with different name
# or in a different directory than the config file.
$ jrny review -c path/to/my-jrny-config.toml -e path/to/my-jrny-env.toml

# Specifying database URL within project directory & default config filename.
# Can be used in conjunction with custom config and/or environment file paths.
#
# If there is a default environment file in the current directory, the URL option
# will take precedence over the URL supplied by the environment file.
$ jrny review -d 'postgresql://user:password@host:5432/dbname'
```

This will list all ordered revisions, each with time of creation as well as time of application, if applied to the specified database.

```bash
$ jrny review

The journey thus far:

  [1] my-first-revision
    Created on 30-Mar-2023 09:10:22
    Applied on 30-Mar-2023 09:11:06

  [2] another-revision
    Created on 30-Mar-2023 09:10:32
    Applied on 30-Mar-2023 09:11:06

  [3] YET-another-revision
    Created on 30-Mar-2023 09:27:58
```

Additionally, `jrny` performs several checks during review to guarantee that...

- Applied revisions' files have not been changed after having been applied
- Applied revisions' files have not been removed
- Pending revisions have not been inserted into the sequence prior to applied revisions
- All revisions in the sequence (pending and applied) have unique ids

```bash
The journey thus far:

  [1] revision-that-gets-changed
    Created on 30-Mar-2023 09:31:05
    Applied on 30-Mar-2023 10:17:33
    Errors:
      - File has changed after being applied

  [2] revision-that-gets-removed
    Created on 30-Mar-2023 09:57:56
    Applied on 30-Mar-2023 10:17:33
    Errors:
      - File could not be found

  [3] a-revision-added-in-between
    Created on 30-Mar-2023 10:18:58
    Errors:
      - Later revisions have already been applied

  [4] some-revision-that-is-fine
    Created on 30-Mar-2023 09:58:19
    Applied on 30-Mar-2023 10:17:33

  [5] a-revision-that-was-fine
    Created on 30-Mar-2023 10:17:24
    Applied on 30-Mar-2023 10:17:33
    Errors:
      - Revision has a duplicate id

  [5] revision-with-duplicate-id
    Created on 30-Mar-2023 10:19:57
    Errors:
      - Revision has a duplicate id

The journey has problems:
  - 1 revision has been changed after being applied
  - 2 revisions have duplicate ids
  - 1 revision file could not be found
  - 1 pending revision has been inserted before revisions already applied
```

These checks are not necessarily mutually-exclusive, either, meaning a single revision
can potentially have multiple errors, eg. having been changed after being applied AND
having a duplicate id, if the sequence has been altered as well.

**Note:** Review will fail with even the addition (or removal) of whitespace or comments;
there is currently no attempt to scrub those out prior to generating the checksums used to
determine if a file has been changed after being applied.

#### Embark on the journey!

To apply all pending revisions, run `jrny embark`.

As with `jrny review`, applying revisions looks for default config & environment files in the current directory,
but either can be overridden and, again, the database URL can be supplied directly.

Revisions will be reviewed prior to applying any pending, and if files have changed, are no longer
present on disk, etc., then `jrny` will issue an error and exit without applying any new revisions.

Otherwise, `jrny` will simply either list the names of the revisions applied...

```bash
$ jrny embark

Applying 1 revision(s)

  003.1680182878.YET-another-revision.sql
```

... or a message indicating that no pending revisions were found.

```bash
$ jrny embark

No revisions to apply
```

Additionally, instead of applying all pending revisions, you can apply only those
up through a specified id using `--through` or `-t`.

For instance, given a review like:

```bash
$ jrny review

The journey thus far:

  [1] my-first-revision
    Created on 30-Mar-2023 09:10:22
    Applied on 30-Mar-2023 09:11:06

  [2] another-revision
    Created on 30-Mar-2023 09:10:32

  [3] YET-another-revision
    Created on 30-Mar-2023 09:27:58

  [4] shocker-a-revision
    Created on 19-Apr-2023 15:42:29

  [5] surprise-another-revision
    Created on 19-Apr-2023 15:42:36
```

If you only wanted to run up through `YET-another-revision` you would just pass the id `3`:

```bash
$ jrny embark --through 3

Applying 2 revision(s), skipping 2

  002.1680181832.another-revision.sql
  008.1681952321.YET another revision.sql
```

## Library Usage

The `jrny` CLI tool is a thin wrapper around several structs and functions that can
alternatively be imported into a Rust application, if one wants to manage revisions
more programmatically.

The library functions make no assumptions about configuration and environment, however;
you must explicitly create those objects as necessary, which is admittedly
not very ergonomic at the moment.

For a complete (basic) example:

```rust
use std::env;
use std::path::PathBuf;

use jrny::context as ctx;


fn main() {
    // Initialize a new `jrny` setup in the `./jrny-test` subdirectory.
    //
    // Note: In addition to creating the necessary revisions directory, this *also*
    // creates the `jrny.toml`, etc files that, when using `jrny` as a library,
    // are entirely unnecessary.
    //
    // See: https://github.com/kevlarr/jrny/issues/35
    jrny::begin(&PathBuf::from("jrny-test")).unwrap();

    // The rest of the commands will need to know the project configuration
    // and potentially other environment details as well.
    let cfg = ctx::Config {
        revisions: ctx::RevisionsSettings {
            directory: PathBuf::from("jrny-test/revisions"),
        },
        table: ctx::TableSettings {
            schema: "public".to_owned(),
            name: "jrny_revision".to_owned(),
        },
    };
    let env = ctx::Environment::from_database_url(&env::var("DATABASE_URL").unwrap());

    // Create a new empty migration
    jrny::plan(&cfg, "my first migration", None).unwrap();

    // Create another migration with some contents
    jrny::plan(&cfg, "a more useful migration", Some("
        create table my_cool_table (
            id bigint
                primary key
                generated always as identity
        )
    ")).unwrap();

    // Review the migrations
    jrny::review(&cfg, &env).unwrap();

    // Run the migrations
    jrny::embark(&cfg, &env).unwrap();
}
```

## Planned improvements, or "things that are missing"

See [enhancements](https://github.com/kevlarr/jrny/issues?q=is%3Aopen+is%3Aissue+label%3Aenhancement)
for a running list of planned new features.

More importantly, there is currently zero test coverage; fixing this is part of the
[v2.0.0 milestone](https://github.com/kevlarr/jrny/issues?q=is%3Aopen+is%3Aissue+milestone%3Av2.0.0).
