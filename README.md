# jrny

Data management is a journey - manage yours with `jrny`.

`jrny` is a prototype PostgreSQL<sup>1</sup> schema-management tool, intended to offer
a free, flexible, native, **opinionated**, and SQL-based alternative to contemporary
(and sometimes proprietary) migration tools that execute changes via Python, Java, etc.
or via YAML files.

> <sup>1</sup> While Postgres is the only current target, the attempt is to design `jrny`
> in a way that proves reasonably extensible to other databases in the future.

## Primary Opinions

### Plain SQL

DSLs, query builders, YAML files, etc. can be convenient, but..

... they require **additional cognitive overhead**, both when learning and when writing migrations
compared to if we JUSQL ("Just Use SQL").

... writing any non-trivial statement in SQL is generally minutes (or hours) faster than attempting the same
statement via a query builder's almost-arcane methods

Basically, since we already know SQL, let's keep things simple (if sometimes verbose) and JUSQL.

### User should decide on transactions

Some ORMs and migration tools decide for themselves whether or not (and even how) migrations
should be run within transactions, and that decision is often difficult to override.

By using raw SQL, the user can manage transactions during migrations in the standard way: `begin` and `commit`.

### Time moves in one direction

Down-migrations are often useful while writing and testing migrations, but they often shouldn't be
relied upon in production code. (It's trivially easy to miss something, like an index on a column being
added back in during a down migration, etc.)

If something isn't easy or safe or reliable in prodution environments, then why use them locally?

### Migrations are best separated from application deploys

If a migration can break application code, then a successful migration and failed deploy will result
in a broken application. Even if deploy succeeds, there is still a lag (however miniscule) between a
migration completing and an application being deployed and restarted.

Zero-downtime deploys are a dream; they're much easier to realize if migrations can't put the application
in an inconsistent or broken state.

This can make data migrations more cumbersome or require more steps (eg. migration -> data migration via app -> migration)
but in the long run it helps to mitigate risk and add more time for testing and verification.

### Compiled binaries are easier to manage than language runtimes

Language runtimes, dynamically-typed or static, are great. There's nothing wrong with them.

But if database migrations can be easily run via a single, easily managed binary rather than an
entire runtime with dependencies that need to be installed, isn't that simpler?


## Usage

Journey can interact with any directory that includes a config file and revision directory.


```bash
# Set up a new project...
$ jrny start path-to-new-project

# ... or from existing
$ cd path-to-project

# Generate a timestamp/keyed SQL file
$ jrny revise create-admin-table

# Inspect current version and determine which to apply
$ jrny review
$ jrny review --budget

# Apply all migrations
$ jrny embark
$ jrny embark --step
```

## Planned improvements, or "the things that are missing"

### Tests

No description necessary.

### Actual docstrings

Doc strings like this...

```rust
/// This is gross but it does things.
```

I mean, just no.

### Better output

The output isn't very descriptive (at all), and there aren't any pretty colors.
In fact, the output often doesn't even really confirm much of what's happened.

### Error handling

The error handling is comically ugly.
There are no custom types or anything classy like that yet,
everything just gets the ol' `.map(|e| e.to_string())?` and `Result<(), String>` treatment.
