# jrny

Data modeling is a journey - manage yours with `jrny`.

`jrny` is a prototype PostgreSQL<sup>1</sup> schema-management tool, intended to offer
a free, flexible, native, SQL-based alternative to contemporary (and sometimes proprietary)
migration tools that execute changes via Python, Java, etc. or via YAML files.

> <sup>1</sup> While Postgres is the only current target, it could theoretically be extended
> in the future to support other databases, though extensibility is not an immediate goal.

## Primary Opinions

### Plain SQL

DSLs, query builders, YAML files, etc. can be convenient, but they require **additional cognitive overhead**
that quickly proves an obstacle the moment one tries to translate a non-trivial SQL statement.

If we already know SQL, why not just use SQL? Having to know just one thing is better than having to know two
things, even if the price is (sometimes) a little extra verbosity.

Being language- and project- independent is actually a good thing, too, as it encourages a
separation of concerns.

### Time moves in one direction

Down-migrations can be useful while writing and testing migrations, but it's trivially easy to miss something,
like an index on a column being added back in during a down migration.
If it was supposed to be that easy to mess up history, we'd probably already all be time-travelers.

If something isn't easy or safe or reliable in prodution environments, then why bother with them locally?

### Simpler is easier and easier is better

Migration tools don't need complicated methods for history tracking or determining the sequence of revisions -
timestamps are good enough. If branches with different revisions get merged together,
then a test database will tell us if there's a problem.

### Compiled binaries are easier to manage than language runtimes

Language runtimes, dynamically-typed or static, are great. There's nothing wrong with them.

But if database migrations can be easily run via a single binary rather than an
entire runtime with dependencies that need to be installed, isn't that simpler?

And isn't simpler better?

## Usage


```bash
# Set up new config and revisions directory
jrny begin <path>

# Generate a new timestamped revision, either by providing
# path to config file...
jrny revise some-revision-name -c <path>/jrny.toml

# ... or from within the project directory
cd <path>
jrny revise 'use quotes for names with spaces'

# View the list of current revisions and see which have been applied and when.
# Additionally, this step will compare checksums to make sure that the file
# hasn't been changed after having been applied.
#
# Again, either specify config file...
jrny review -c <path>/jrny.toml

# ... or enter into project directory
cd <path>
jrny review
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
