---
description: Implement and complete a single task.
argument-hint: [task-file]
---

study the task in $ARGUMENTS.
study the task's spec in specs/{spec-name}/README.md.

1. Your job is to implement the task per the spec. Only complete that one task, and then stop (don't try to do too much in one session). Before making changes, search the codebase (don't assume not implemented).
1b. Update specs/README.md and specs/{spec-name}/tasks/README.md to show that the spec & task are In Progress (always keep these files updated!).
2. Follow the Software Development process.
3. When the task is complete, update specs/README.md and specs/{spec-name}/tasks/README.md and any checklists in specs/{spec-name}/tasks/{task.md} with the proper status (In Progress or Complete) (always keep those files updated!)
4. Commit all changes by simply running `scripts/commit.sh <message>` with a very brief, fast commit messages of changes. You must not use any git commands to generate that message, just quickly summarize what you did. Speed is essential.
5. Before finishing up, make sure any servers or docker containers you ran are stopped/killed.
6. After the task and all TODOs are complete, Claude always wants to provide a summary of what was accomplished. However, it takes way too long. You must not summarize at the very end. Just state the task is complete, and then stop.

9999. VITAL: you must keep the spec and task statuses updated at all times! Future sessions won't know what to do unless you keep statuses updated. Spec status is in specs/README.md and task status is in specs/{spec-name}/tasks/README.md.
99999. Important: When authoring documentation, capture the why — tests and implementation importance.
999999. Important: Single sources of truth, no migrations/adapters. If tests unrelated to your work fail, resolve them as part of the increment.
99999999. You may add extra logging if required to debug issues.
999999999. Keep `specs/{spec-name}/*` current with learnings using a subagent — future work depends on this to avoid duplicating efforts. Update especially after finishing your turn.
9999999999. When you learn something new about how to run the application, update @CLAUDE.md using a subagent but keep it brief. For example if you run commands multiple times before learning the correct command then that file should be updated.
999999999999. Implement functionality completely. Placeholders and stubs waste efforts and time redoing the same work.
999999999999999. IMPORTANT: Keep @CLAUDE.md operational only — status updates and progress notes belong in `specs/{spec-name}/*`. A bloated CLAUDE.md pollutes every future loop's context.
