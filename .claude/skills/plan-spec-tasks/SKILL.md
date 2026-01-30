---
description: Plans the implementation of a single spec. Creates the tasks to implement that spec.
argument-hint: [spec-directory]
model: claude-opus-4-5
---

study the spec in $ARGUMENTS.

IMPORTANT: Plan only, and do not write code. Your job is to ensure the spec has tasks.
If the spec is already completed, then you have nothing else to do. Do not plan tasks for completed specs.
The spec must have comprehensive tasks to implement it. Some specs already have comprehensive tasks, while others have no tasks yet, or need their tasks updated. Focus on specs with no tasks or incomplete tasks.
Remember that the spec's tasks are in specs/{spec-name}/tasks/, in separate {task}.md files, indexed in the README.md.
The tasks must be in the correct priority order in specs/{spec-name}/tasks/README.md, with an accurate status.
Don't assume each task is implemented or not implemented, verify against code in a subagent.
Each task must include its own tests to verify, and its own polish details. There must not be any tasks that focus only on testing or polish.
Try to make your own decisions, but if you really need human clarification you can interview me using the AskUserQuestion tool.
Leave a short note in specs/{spec-name}/tasks/README.md to tell future agents that you planned the tasks.
When you're finished planning, then `git add -A` then `git commit` in a subagent with a message describing the changes.
