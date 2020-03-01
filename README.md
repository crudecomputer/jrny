# jrny

Journey is a prototype Postgres schema management tool, intended to offer
a free, flexible, native, opinionated, and SQL-based alternative to contemporary
migration tools that execute changes via Python, Java, etc.

Primary opinions include...

- Plain SQL - DSLs are nice but hard to use for anything complicated
- History is one-way - no down migrations
- Compiled applications are easier to manage than language runtimes
- Database management should be isolated from applications


## Proposed features

- One-way
- Plan feature with proposed costs, a la Terraform
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
