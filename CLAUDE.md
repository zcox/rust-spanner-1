## Specs, Tasks, Priority, and Status

Specs describe what the system does, and why. Each spec describes a single topic of concern for the system.

Specs are found in the specs/ directory, indexed in the specs/README.md file. Each spec is a subdirectory, with the spec in its specs/{spec-name}/README.md file.

Tasks describe how each spec will be implemented. Each spec has a tasks subdirectory with a specs/{spec-name}/tasks/README.md file that links to each task in that directory. The specs/{spec-name}/README.md file must always link to that spec's tasks file at specs/{specs-name}/tasks/README.md.

Priority defines the order of task implementation. The highest priority task should always be worked on next. specs/README.md always lists specs in priority order, and specs/{spec-name}/tasks/README.md always lists tasks in priority order.

Specs and tasks both have a current status that is always up-to-date:
  - 🔲 Not started
  - 🔄 In progress
  - ✅ Complete

└── specs
    ├── README.md
    └── spec-name
       ├── README.md
       └── tasks
          ├── README.md
          └── task.md

## Software Development

1. generate code for task.
2. verify the code lints, compiles, builds, etc. go to 1 until verified.
3. generate appropriate tests (unit, property-based, integration, etc) for the code.
4. verify that all tests pass, go to 3 until verified.
5. run the app.
6. verify that the code functions correctly (use curl/jq/etc to verify endpoint, verify client changes, etc), go to 1 until verified.

## Verification

Run these after implementing to verify changes:

- Lint: `[lint command]`
- Compile: `[compile command]`
- Tests: `[test command]`
- Run: `[run command]`
