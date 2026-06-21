# Efforts and components

Both are optional. Enable them in `schema/features.yaml` (and mirror in
`track.yaml`) when your project needs them.

## Efforts

An **effort** groups issues for focused progress tracking. The same mechanism
supports:

- Software sprints or milestones
- Animation deliveries or reels
- Renovation phases or trip segments

Efforts are **temporal or goal-oriented**—they answer *when* or *which wave* of
work. Issues link to an effort via the `effort` field (a URN reference).

Efforts can depend on other efforts (`blocks`, `requires`) for roadmap-style
planning.

## Components

A **component** represents a structural artifact within the project:

- Software subsystem or service
- Animation scene or character rig
- Room or trade in a renovation
- PCB block or firmware module

Components are **structural**—they answer *what part* of the deliverable.
Issues may reference both an effort and a component: "Install outlets" might
belong to **Phase 1** (effort) and **Kitchen** (component).

## When to enable each

| Project style | Efforts | Components |
| --- | --- | --- |
| Personal todo list | Off | Off |
| Software team | On (sprints) | On (services/repos) |
| Animation production | On (deliveries) | On (sequences/scenes) |
| Home renovation | On (phases) | On (rooms/trades) |

The [schema examples](../schema/examples/software-project.md) show typical
combinations for each domain.
