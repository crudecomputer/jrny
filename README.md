# jrny

*Data modeling is a journey - manage yours with `jrny`*

---

**Disclaimer:** As far as I am aware, I am the only one using this tool, and that's only on hobby projects.
While it is functional for personal use cases, it is far from battle-tested. Or even unit/integration tested.
**Use at your own risk.**

---

## Overview

A lot of schema-migration tools already exist. They work, and they work pretty well at that,
but there are (subjectively) some big annoyances and I just don't like them.

`jrny` offers an alternative for people who...

* ... think database revision files should be an 'immutable' record and are guaranteed to represent what was applied to database

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

### Pre-compiled

`jrny` is currently only pre-compiled for mac and can be installed either via direct download from [releases](https://github.com/kevlarr/jrny/tags) or via `homebrew`:

```bash
homebrew install kevlarr/jrny/jrny
```

### From source

While `jrny` has only been fully tested on macOS (CI runs unit tests on Ubuntu, but no tests currently interact with filesystem or database), there are zero external dependencies and it
*Should Just Work™* on other platforms, as long as you compile it yourself.
Assuming `cargo` is installed (easiest is using [rustup](https://rustup.rs/)) then simply run:

```bash
$ cargo install jrny

# Sample output on macOS
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
```

The default `jrny.toml` file specifies how `jrny` will connect to the target database, for instance:

```
[connection]
strategy = { type = "env-url-string", var-name = "JRNY_DATABASE_URL" }

[table]
schema = "public"
name = "jrny_revision"
```

This means that `jrny` will look for a connection string in the `JRNY_DATABASE_URL` environment variable
and store revision history in the `public.jrny_revision` table. The variable name can be changed any time
but, once revisions are applied and tracked, the schema and table name should not be changed, as `jrny`
would then attempt to apply all revisions from the beginning.

The config file can be freely renamed if desired; however, this requires passing the filepath in via `-c` to all commands.

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
particularly if you need to execute statements outside of a transaction.

For now, there are no checks that edits to the file keep the matching `begin;` and `commit;` commands.

``` bash
$ cat /path/to/my/revisions/002.1606743400.name\ with\ spaces.sql
-- 002.1606743400.name with spaces.sql

begin;
-- Start revisions


-- End revisions
commit;
```

Revision filenames follow the pattern of `[id].[timestamp].[name].sql`.

Timestamps are just great metadata to capture, and `jrny` assigns a sequential id to each file.
The reason being this enforces a stricter revision order than simply using timestamps can,
all without needing pointers between files.
(For more information, see https://github.com/kevlarr/jrny/issues/17)

Gaps in the id sequence are fine (eg. if you create two new revisions, remove the first one, and then apply the second),
and ids can be manually changed as long as the revision hasn't been applied.

### Review the journey

To summarize the state of revisions, run `jrny review [-c <path-to-config>]`, again either specifying the config
file to use or defaulting to looking in current directory.

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

To apply pending revisions, run `jrny embark [-c <path-to-config>]`.

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

Refactoring in Rust is fun - which is good, because there's a lot of room for clearer patterns and modules.

### Revision archiving

Revisions are great, but we don't normally need revisions from 2 years ago sitting in the directory.
There should be a command to help 'reset' history, potentially archiving them into a table..?

### Revision 'bundling'

Sometimes revisions are logically-related (ie. when developing a given feature) and it could make
sense to group them together in a folder, just to help keep the files a little easier to browse.

### Tests and automation

No description necessary; there's barely any test coverage, and there's hardly any CI.
