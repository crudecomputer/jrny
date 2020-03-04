# jrny

Journey is a prototype Postgres schema management tool, intended to offer
a free, flexible, native, opinionated, and SQL-based alternative to contemporary
migration tools that execute changes via Python, Java, etc.

Primary opinions include...

- Plain SQL: DSLs are nice but hard to use for anything complicated
- History is one-way: down migrations get complicated and are very easy to miss something (eg. an index)
- Compiled applications are easier to manage than language runtimes
- Encourage stress-free revisions: database management should be untangled from application deploys


## Proposed features

- Plan feature with estimated costs for each migration
- Live progress (statements executed, active vs. blocked, etc)
- Revision statistics in table
- Explicit commit confirmation
- Schema output file
- Schema validation relative to database


## Use

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
