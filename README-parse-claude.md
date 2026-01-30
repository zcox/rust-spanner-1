# Claude Output Parser

Parse Claude CLI JSON output (`--output-format stream-json`) to show only the important bits.

## Usage

```bash
# Pipe from Claude CLI
cat PROMPT.md | claude -p --output-format stream-json | ./parse-claude

# Or parse existing output file
./parse-claude output-build

# Or from stdin
./parse-claude < output.json
```

## Modes

### Compact Mode (default)
Shows only assistant messages and tool names - great for quick overview:

```bash
./parse-claude output-build
# or explicitly:
./parse-claude --compact output-build
```

Output:
```
[ASSISTANT] I'll start by studying the implementation plan...
[→] Glob
[→] Read: RideCard.tsx
[→] Write: RequestCard.tsx
[→] Bash: TypeScript type check for frontend
[⚡] Subagent: general-purpose - Update implementation plan
```

### Normal Mode
Shows assistant messages, tool details (commands, descriptions), and results:

```bash
./parse-claude --normal output-build
```

Output:
```
[ASSISTANT] I'll start by studying the implementation plan...
[TOOL] Glob: **/RequestCard.tsx
[RESULT] No files found

[TOOL] Bash: TypeScript type check for frontend
  Command: cd frontend && npx tsc --noEmit
[RESULT]

[SUBAGENT] general-purpose: Update implementation plan
  Prompt: Mark the RequestCard component as complete...
```

## Color Legend

- **Cyan** `[ASSISTANT]` - Claude's messages
- **Yellow** `[TOOL]` / `[→]` - Tool calls (Read, Write, Edit, Bash, etc.)
- **Magenta** `[SUBAGENT]` / `[⚡]` - Subagent Task calls
- **Green** `[RESULT]` - Tool results (normal mode only)

## Examples

```bash
# Stream live output (compact)
cat PROMPT.md | claude -p --output-format stream-json | ./parse-claude

# Stream live output (normal)
cat PROMPT.md | claude -p --output-format stream-json | ./parse-claude --normal

# Parse saved output with less paging
./parse-claude output-build | less -R

# Search for specific assistant messages
./parse-claude output-build | grep ASSISTANT

# Count how many tools were called
./parse-claude --compact output-build | grep "^\\[→\\]" | wc -l
```

## Help

```bash
./parse-claude --help
```

## Files

- `parse-claude` - Main script with mode selection
- `parse-claude-compact.jq` - jq filter for compact mode
- `parse-claude-output.jq` - jq filter for normal mode
