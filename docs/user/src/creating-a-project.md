# Creating a project

This chapter walks through initializing a Track project and validating its
schema. Commands assume the `track` binary is on your `PATH` (built from this
repository or installed elsewhere).

## Prerequisites

- A directory for the project (empty repo, existing repo, or standalone folder)
- Optional: a workspace hub URL or slug (defaults to `personal` for local dev)

User identity (`node_uuid`, default actor) is created automatically on first
run under your platform config directory.

## Initialize

Create a project with the built-in **default** template:

```bash
# Standalone project in the current directory
track init MYAPP --standalone

# Embedded layout: ./track/ inside a source repository
cd my-repo
track init MYAPP

# Explicit name and workspace
track init KITCHEN --name "Kitchen Renovation" --workspace personal
```

`track init` writes:

- `track.yaml` manifest with a new `project_uuid`
- `schema/` from the template
- `.track/` local state directory
- Suggested `.gitignore` rules for lazy `work/` directories

### Re-initialize

Pass `--force` to replace schema files from the template while **preserving**
`project_uuid`. Use this to reset a botched schema edit—not for routine updates.

## Validate schema

After tailoring `schema/*.yaml`, check cross-file consistency:

```bash
cd my-project   # or use --project PATH from anywhere
track schema validate
```

A successful run prints nothing to stdout (unless `--json`). Errors include
file, path, code, and message for each issue.

## Plan a push (dry run)

Preview hub events without contacting a server:

```bash
track push --dry-run
```

With `--debug`, the CLI logs full event envelopes to stderr. Live push (without
`--dry-run`) requires a configured hub and is still under development.

## Next steps

- Read [Tailoring your schema](./schema/tailoring.md) and pick a domain
  [example](./schema/examples/software-project.md)
- Use the [schema reference](./schema/reference/states.md) while editing YAML
- Commit `schema/` and `track.yaml`; lazy `work/` dirs may stay gitignored
