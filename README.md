# jrny

*Data modeling is a journey - manage yours with `jrny`*

## Overview

A lot of schema migration tools already exist,
but there is still room for others.

`jrny` offers an alternative for people who...

* ... think database revision files should be an immutable record and are **guaranteed to represent** what was applied to database

* ... would **rather write SQL** than translate it to method calls or YAML entries that are often more verbose and less documented

* ... prefer to **install compiled binaries** rather than manage a language and dependencies on whatever system(s) run migrations

* ... like the idea of **single responsibility**, especially if multiple applications (potentially in different repos and written in different languages) access the same tables

* ... believe that **separating migrations from application deploys** encourages one to write non-breaking migrations and helps enable zero-downtime updates

* ... think **down-migrations are unnecessary**<sup>1</sup> at best and dangerous at worst, especially as they make it trivially easy to 'change' history by forgetting to add a preexisting index, check constraint, etc. during the upgrade/downgrade/edit/upgrade cycle.


<small>

> <sup>1</sup> Down-migrations are great for iteratively developing complex schema changes,
> as it lets the developer make a small change, roll it back, add another change, etc. until
> the migration performs as intended.
>
> *One alternative*, especially for those of us who prefer SQL,
> is to **iterate on statements within a transaction** in another client (eg. `psql`) and then just
> add the final statements to a revision & apply. Using a transaction and another client to test SQL
> statements *also* means the developer can explore schema changes interactively. *Rollbacks are great, m'kay.*
>
> A *second* alternative is to drop the local database and re-create it if a revisions needs
> to be adjusted. This strongly encourages setting up seed files, too, which is huge for
> having easily set-up developer environments.

</small>

---

## Installation

### From source

Assuming `cargo` is installed (easiest is using [rustup](https://rustup.rs/)) then simply run:

```bash
$ cargo install jrny --version 2.0.0-beta.4

    Updating crates.io index
  Downloaded jrny v1.3.0
  Downloaded 1 crate (28.6 KB) in 0.39s
  Installing jrny v1.3.0
   ...
   ...
   ...
   Compiling jrny v1.3.0
    Finished release [optimized] target(s) in 2m 03s
  Installing /Users/<user>/.cargo/bin/jrny
   Installed package `jrny v1.3.0` (executable `jrny`)
```

---

## Usage

There are **4 steps** to managing schema changes with `jrny`:

1. `begin`
2. `plan`
3. `review`
4. `embark`

### Begin the journey

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

### Plan the journey

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
(For more information, see https://github.com/kevlarr/jrny/issues/17)

Gaps in the id sequence are fine (eg. if you create two new revisions, remove the first one, and then apply the second),
and ids can be manually changed as long as the revision hasn't been applied.

### Review the journey

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

The journey thus far

  Id   Revision                                   Created                  Applied
    1  my first revision                          13-Apr-2021 23:18:18     15-Apr-2021 22:19:07
    2  another-change                             14-Apr-2021 21:22:43     15-Apr-2021 22:19:07
    3  yet-another-change                         14-Apr-2021 21:42:34     --
```

Additionally, `jrny` performs several checks during review to guarantee that...

#### ... all applied revisions are still present on disk

If any revision files that have been applied are removed, review will fail with...

```bash
$ jrny review

The journey thus far

  Id   Revision                                   Created                  Applied
    1  my first revision                          13-Apr-2021 23:18:18     13-Apr-2021 23:29:37
    2  another-change                             14-Apr-2021 21:42:34     14-Apr-2021 22:32:35     No corresponding file could not be found
```

#### ... all applied revision files have not changed since application (compared by SHA-256 checksum)

Guaranteeing that revision files are still all present isn't useful without an additional
guarantee that they haven't *changed* since being applied.

```bash
$ jrny review

The journey thus far

  Id   Revision                                   Created                  Applied
    1  my first revision                          13-Apr-2021 23:18:18     15-Apr-2021 22:22:23
    2  another-change                             14-Apr-2021 21:22:43     15-Apr-2021 22:22:23     The file has changed after being applied
```

**Note:** This will fail with even the addition of whitespace or comments; there is currently
no attempt to scrub those out prior to generating the checksums.

#### ... all revisions have unique ids

Self-explanatory; an id isn't much of an id if it isn't unique.
This is performed prior to applying to database,
so that if there are 3 revisions to run and 1 has a duplicate id,
no revisions will be attempted.

```bash
$ jrny review

The journey thus far

  Id   Revision                                   Created                  Applied
    1  my first revision                          13-Apr-2021 23:18:18     --
    1  another-change                             14-Apr-2021 21:42:34     --                       Revision has duplicate id
```

#### ... no unapplied revisions can occur earlier in the sequence than applied ones

```bash
The journey thus far

  Id   Revision                                   Created                  Applied
    1  my first revision                          13-Apr-2021 23:18:18     13-Apr-2021 23:29:37
    2  another-change                             14-Apr-2021 21:42:34     --                       Later revisions have already been applied
    3  yet-another-change                         14-Apr-2021 21:22:43     14-Apr-2021 21:37:17
```

### Embark on the journey!

To apply pending revisions, run `jrny embark`.

As with `jrny review`, applying revisions looks for default config & environment files in the current directory,
but either can be overridden and, again, the database URL can be supplied directly.

Revisions will be reviewed prior to applying any pending, and if files have changed, are no longer
present on disk, etc., then `jrny` will issue an error and exit without applying any new revisions.

For instance, combining the examples of failed review above, you might see any or all of the following
when attempting to embark.

```bash
$ jrny embark

Error: Failed to run revisions:
	1 changed since being applied
  1 pending occur before applied revisions
	1 no longer present on disk
  1 has a duplicate id
```

If the files were restored, changes re`jrny` would move forward with applying `1606749809.so many things.sql` and you would instead see...

If the errors were resolved...

* changed files were reverted
* missing files were restored
* duplicate ids were fixed
* unapplied revisions 'moved' after applied ones (by adjusting ids)

... then you would see successful revision application.

```bash
$ jrny embark

Applying 4 revision(s)

  001.1618370298.my first revision.sql
  003.1618449763.another-change.sql
  004.1618450954.YET-another-change.sql
  005.1618370304.my first revision.sql
```

Attempting to apply revisions again would simply find none available.

```bash
$ jrny embark

No revisions to apply
```

---

## Planned improvements, or "things that are missing"

### Code cleanup

Refactoring in Rust is fun - which is good, because there's a lot of room in this project
for clearer patterns and modules, better code, etc.

### Revision archiving

Revisions are great, but we don't normally need revisions from 2 years ago sitting in the directory cluttering up the screen.

This could take several forms but could possible involve concatenating all files into a single 'base' revision and resetting the revision
table to mark that base as applied.

### Revision 'bundling'

Sometimes revisions are logically-related (ie. when developing a given feature) and it could make
sense to group them together in a folder, just to help keep the files a little easier to browse.

### Tests and automation

No description necessary; there's barely any test coverage, and there's hardly any CI.
