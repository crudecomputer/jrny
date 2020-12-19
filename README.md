# jrny

*Data modeling is a journey - manage yours with `jrny`*

---

## Overview

A lot of schema-migration tools already exist. They work, and they work pretty well at that,
but there are (subjectively) some big annoyances and I just don't like them.

`jrny` offers an alternative<sup>1</sup> for people who...

* ... would **rather write SQL** than translate it to method calls or YAML entries that are often more verbose and less documented

* ... prefer to **install compiled binaries** rather than manage a language and dependencies on whatever system(s) run migrations

* ... like the idea of **single responsibility**, especially if multiple applications (potentially in different repos and written in different languages) access the same tables

* ... believe that **separating migrations from application deploys** encourages one to write non-breaking migrations and helps enable zero-downtime updates

* ... think **down-migrations are unnecessary**<sup>2</sup> at best and dangerous at worst, especially as relying on them during development makes it trivially easy to 'change' history by forgetting to add an index, check constraint, etc.


<small>

> <sup>1</sup> `jrny` is still in a prototype state and currently **only supports PostgreSQL**.
> While it could theoretically be extended fairly easily in the future to support other databases,
> extensibility is not an immediate goal.
>
> <sup>2</sup> Down-migrations are great for iteratively developing complex schema changes,
> as it lets the developer make a small change, roll it back, add another change, etc. until
> the migration performs as intended. *An alternative*, especially for those of us who prefer SQL,
> is to **iterate on statements within a transaction** in another client (eg. `psql`) and then just
> add the final statements to a revision & apply. Using a transaction and another client to test SQL
> statements *also* means the developer can explore schema changes interactively. *Rollbacks are great, m'kay.*

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

Created revisions/1606743300.create-users.sql

$ jrny plan 'name with spaces' -c /path/to/my/config.toml

Created /path/to/my/revisions/1606743400.name with spaces.sql
```

This will create a (mostly) empty SQL file for you to populate with wonderful statements.
Notice that `jrny` **encourages transactions per-revision** but you are free to remove these,
particularly if you need to execute statements outside of a transaction.

``` bash
$ cat /path/to/my/revisions/1606743400.name with spaces.sql
-- 1606743400.name with spaces.sql

begin;
-- Start revisions


-- End revisions
commit;
```

### Review the journey

To summarize the state of revisions, run `jrny review [-c <path-to-config>]`, again either specifying the config
file to use or defaulting to looking in current directory.

This will list all ordered revisions, each with time of creation as well as time of application, if applied to the specified database.
Additionally, `jrny` will also ensure that...

* ... all applied revisions are still present on disk

* ... all applied revision files have not changed since application (compared with SHA-256 checksum)

```bash
$ jrny review

The journey thus far

Revision                                          Created                  Applied
1606743368.create-users.sql                       30-Nov-2020 08:36:08     30-Nov-2020 08:38:43     The file has changed after being applied
1606749730.another revision.sql                   30-Nov-2020 10:22:10     30-Nov-2020 10:23:12
1606749751.drop-things.sql                        30-Nov-2020 10:22:31     30-Nov-2020 10:23:12     No corresponding file could not be found
1606749776.create-more-things.sql                 30-Nov-2020 10:22:56     30-Nov-2020 10:23:12
1606749809.so many things.sql                     30-Nov-2020 10:23:29     --
```

### Embark on the journey!

To apply pending revisions, run `jrny embark [-c <path-to-config>]`.

Revisions will be reviewed prior to applying any pending, and if files have changed or are no longer
present on disk, `jrny` will issue an error and exit without applying any new revisions.

For instance, given the history above, you would see...

```bash
$ jrny embark

Error: Failed to run revisions:
	1 changed since being applied
	1 no longer present on disk
```

If the files were restored and reverted, `jrny` would move forward with applying `1606749809.so many things.sql` and you would instead see...

```bash
$ jrny embark

Applying 1 revision(s)

  1606749809.so many things.sql
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

### Tests and automation

No description necessary; there's barely any test coverage, and there's hardly any CI.
