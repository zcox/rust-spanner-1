---
description: Claude interviews you about your idea and creates specs.
argument-hint: [idea-file]
model: claude-opus-4-5
---

study idea in $ARGUMENTS
study specs/README.md

rename the idea file to something sensible.
start creating new specs based on the idea.
let's have a discussion and you can interview me using the AskUserQuestion tool.

#### Concepts

| Term                    | Definition                                                      |
| ----------------------- | --------------------------------------------------------------- |
| _Job to be Done (JTBD)_ | High-level user need or outcome                                 |
| _Topic of Concern_      | A distinct aspect/component within a JTBD                       |
| _Spec_                  | Specifications doc for one topic of concern (`specs/{spec-name}/README.md`) |
| _Task_                  | Unit of work derived from comparing specs to code (`specs/{spec-name}/tasks/`) |

_Relationships:_

- 1 JTBD → multiple topics of concern
- 1 topic of concern → 1 spec
- 1 spec → multiple tasks (specs are larger than tasks)

_Example:_

- JTBD: "Help designers create mood boards"
- Topics: image collection, color extraction, layout, sharing
- Each topic → one spec file
- Each spec → many tasks in implementation plan

_Topic Scope Test: "One Sentence Without 'And'"_

- Can you describe the topic of concern in one sentence without conjoining unrelated capabilities?
  - ✓ "The color extraction system analyzes images to identify dominant colors"
  - ✗ "The user system handles authentication, profiles, and billing" → 3 topics
- If you need "and" to describe what it does, it's probably multiple topics
