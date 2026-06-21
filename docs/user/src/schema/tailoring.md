# Tailoring your schema

Templates give you a working baseline. **Tailoring** means editing `schema/`
(and manifest defaults) so issue types, states, and fields match how you
actually work.

## Start from the default template

The built-in `default` template provides:

- One type: **Task**
- States: Backlog → Todo → In Progress → Done / Cancelled
- One workflow binding Task to those states
- Features off: efforts, components, hierarchy, relation enforcement

That is enough for a personal todo list. Most real projects add types, labels,
transitions, and custom properties—and turn on feature flags as needed.

## Edit in order

Follow the [authoring order](../concepts/schema-overview.md):

1. Define **states** and semantic groups (for progress reporting)
2. Add **labels** you expect to filter on
3. Define **workflows** that list states per type
4. Declare **types** with workflows and custom `properties`
5. Set **features** to match (efforts, components, hierarchy, strict relations)
6. Align **`track.yaml`** defaults with your primary type and workflow

Run `track schema validate` after each meaningful batch of edits.

## Domain examples

These chapters sketch full schemas for three domains. They are **starting
points**—copy patterns, not prescriptions:

| Example | Focus |
| --- | --- |
| [Software project](./examples/software-project.md) | Story/Bug/Task, sprints, services, strict relations |
| [Animation project](./examples/animation-project.md) | Shots/assets, deliveries, sequences |
| [Home improvement](./examples/home-improvement.md) | Phases, rooms, purchases and trades |

Each example calls out which files change and shows representative YAML fragments.
Complete file listings will grow as templates for those domains land in the
repository.

## Push when ready

After validation, push schema to the workspace hub (when live sync is
available). Schema changes should go through the same review flow as code—PRs,
agent proposals, and human sign-off all apply.
