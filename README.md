# jrny

Data modeling is a journey - manage yours with `jrny`.


## Overview

A lot of schema-migration tools already exist. They work, and they work pretty well at that,
but there are (subjectively) some big downsides.

`jrny` offers an alternative for people who...

* ... would **rather write SQL** than translate it to method calls or YAML entries that are often more verbose and less documented

* ... prefer to **install compiled binaries** rather than manage a language and dependencies on whatever system(s) run migrations

* ... like the idea of **single responsibility**, especially if multiple applications (potentially in different repos and written in different languages) access the same tables

* ... believe that **separating migrations from application deploys** encourages one to write non-breaking migrations and helps enable zero-downtime updates

* ... think **down-migrations are impractical** at best and dangerous at worst, especially as relying on them during development makes it trivially easy to 'change' history by forgetting to add an index, check constraint, etc.


`jrny` is still in a prototype state and currently **only supports PostgreSQL**.
While it could theoretically be extended fairly easily in the future to support other databases,
extensibility is not an immediate goal.

## Installation

At the moment, `jrny` must be manually built using `cargo`, but this will change in a subsequent release.

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

### Plan the journey

To create a new SQL revision, run `jrny plan [-c <path-to-config>]` either specifying the path to the `jrny.toml`
file via `-c` or by omitting and defaulting to looking in current directory.

This will create an empty SQL file for you to populate with wonderful statements.

```bash
$ jrny plan create-users

Created revisions/1606743300.create-users.sql

$ jrny plan 'name with spaces' -c <path>/jrny.toml

Created <path>revisions/1606743400.name with spaces.sql
```

### Review the journey

To summarize the state of revisions, run `jrny review [-c <path-to-config>]`, again either specifying the config
file to use or defaulting to looking in current directory.

This will list all ordered revisions with time of creation and, if applied, application to database.
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

To apply pending revisions, run `jrny embark [-c <path-to-config> --commit]`.
All statements will be executed *within a single transaction* that, by default, is **rolled back** upon completion
unless explicitly told to commit via the `--commit` flag.
This is to encourage dry-runs, which is particularly helpful while developing migrations -
each statement can be added and tested incrementally, without requiring a down migration to undo changes.

Revisions will be reviewed prior to applying any pending, and if files have changed or are no longer
present on disk, `jrny` will issue an error and exit without applying any new revisions.

For instance, given the history above, you would see...

```bash
$ jrny embark

Error: Failed to run revisions:
	1 changed since being applied
	1 no longer present on disk
```

If the files were restored and reverted, `jrny` would go ahead with applying `1606749809.so many things.sql`.
In the process, it would preview each individual statement as it is being applied. For instance,
if a new multi-statement revision is added...

```
$ jrny plan create-pets

Created revisions/1606753722.create-pets.sql

$ echo "
CREATE TABLE pet (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);

INSERT INTO pet (name) VALUES
  ('Eiyre'),
  ('Cupid'),
  ('My imaginary iguana');
" >> revisions/1606753722.create-pets.sql
```

... then `jrny embark` would output:

```
$ jrny embark

Found 1 revision(s) to apply
	1606753722.create-pets.sql

Applying "1606753722.create-pets.sql"
	CREATE TABLE pet ( id SERIAL PRIMARY KEY, name TEXT NOT NULL ...
	INSERT INTO pet (name) VALUES ('Eiyre'), ('Cupid'), ...

Rolling back the transaction - use `--commit` to persist changes


$ jrny embark --commit

Found 1 revision(s) to apply
	1606753722.create-pets.sql

Applying "1606753722.create-pets.sql"
	CREATE TABLE pet ( id SERIAL PRIMARY KEY, name TEXT NOT NULL ...
	INSERT INTO pet (name) VALUES ('Eiyre'), ('Cupid'), ...

Committing the transaction


$ jrny embark --commit

No revisions to apply
```

**Note:** SQL revisions will **fail to apply** if they include transaction commands like `BEGIN`
or `COMMIT`, because the assumption (for now) is that the transaction should be fully
managed by `jrny`. (If there are good use cases against this, **please** file an issue.)

## Planned improvements, or "things that are missing"

### Revision archiving

Revisions are great, but we don't normally need revisions from 2 years ago sitting in the directory.
There should be a command to help 'reset' history, potentially archiving them into another table somewhere.

### Session exploration

Down-migrations have one BIG advantage during development compared to dry runs:
they leave the database in a state that the developer *can explore* after revisions are applied.

This lets them revert changes, tweak the revision, and re-apply.

It would be a huge bonus if `jrny` somehow offered something similar, perhaps by exposing the active
session to run queries against prior to rolling back and/or committing.

### Tests and automation

No description necessary; there's barely any test coverage, and there's no CI.
