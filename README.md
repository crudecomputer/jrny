# jrny

Data modeling is a journey - manage yours with `jrny`.

`jrny` is a prototype PostgreSQL<sup>1</sup> schema-management tool, intended to offer
a simple, free, native, and SQL-based alternative to contemporary (and sometimes proprietary)
migration tools that execute changes via language runtimes and code scripts or .yml files.

> <sup>1</sup> While Postgres is the only current target, it could theoretically be extended
> in the future to support other databases, though extensibility is not an immediate goal.

## Primary Opinions

### Plain SQL

DSLs, query builders, YAML files, etc. can be convenient, but they require **additional cognitive overhead**
that quickly proves an obstacle the moment one tries to translate a non-trivial SQL statement.

If we already know SQL, why not just use SQL? Having to know just one thing is better than having to know two
things, even if the price is (sometimes) a little extra verbosity.

Being language- and project- independent is actually a good thing, too, as it encourages a
separation of concerns, and untying schema changes from application code and deploys goes *a long way*
toward enabling zero-downtime migrations.

### No down migrations

Down-migrations can be useful while writing and testing migrations,
but it's trivially easy to miss something (like an index on a column being added back in)
or 'change history' (like adding a check constraint that's different).
If the down migrations are used on one database and not another, then they are now out of sync.
If it was supposed to be that easy to mess up history, we'd probably already all be time-travelers.

Rather than down migrations, `jrny` instead encourages dry runs where revisions are applied but not committed by default.

### Simpler is easier and easier is better

Migration tools don't need complicated methods for history tracking or determining the sequence of revisions -
timestamps are good enough. If branches with different revisions get merged together,
then a test database will tell us if there's a problem.

### Compiled binaries are easier to manage than language runtimes

Language runtimes, dynamically-typed or static, are fantastically great tools.

But if database migrations can be easily run via a single (relatively tiny) binary rather than an
entire runtime with dependencies that need to be installed on a remote system, isn't that simpler?

And isn't simpler better?

## Installation

At the moment, `jrny` is only built from scratch via `cargo`.

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

### Installation

Nobody should need `cargo` to build a tool and then copy it to their path somewhere.
It should be built (cross-platform) and be made accessible.

### Tests and automation

No description necessary; there's barely any test coverage, and there's no CI.

### Revision archiving

Revisions are great, but we don't normally need revisions from 2 years ago sitting in the directory.
There should be a command to help 'reset' history, potentially archiving them into another table somewhere.

### Session exploration

Down-migrations have one BIG advantage during development compared to dry runs:
they leave the database in a state that the developer *can explore* after revisions are applied.

This lets them revert changes, tweak the revision, and re-apply.

It would be a huge bonus if `jrny` somehow offered something similar, perhaps by exposing the active
session to run queries against prior to rolling back and/or committing.