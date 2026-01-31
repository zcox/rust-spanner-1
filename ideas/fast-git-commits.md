# Optimize git commits

at the end of implement-next-task skill are instructions to git add & commit changes quickly with a brief summary. but it takes many seconds to do, and is way too slow. here are two example outputs:

## Example 1

[+2s] [23:30:54] in:0 cache_read:50917 cache_create:409 out:1
[ASSISTANT] Now let me commit the changes.

[+2s] [23:30:56] in:0 cache_read:50917 cache_create:409 out:1
[⚡] Subagent: Bash - Commit health endpoint changes

[+2s] [23:30:58] in:3 cache_read:3426 cache_create:1583 out:5
[→] Bash: Show working tree status

[+1s] [23:30:59] in:3 cache_read:3426 cache_create:1583 out:5
[→] Bash: Show unstaged changes

[+1s] [23:31:00] in:3 cache_read:3426 cache_create:1583 out:5
[→] Bash: Show recent commit messages

[+5s] [23:31:05] in:7 cache_read:5009 cache_create:3221 out:1
[→] Bash: Stage changes and create commit with co-authored-by line

## Example 2

[+2s] [23:34:53] in:0 cache_read:52441 cache_create:306 out:1
[ASSISTANT] Now let me commit the changes.

[+2s] [23:34:55] in:0 cache_read:52441 cache_create:306 out:1
[⚡] Subagent: Bash - Commit error handling implementation

[+2s] [23:34:57] in:3 cache_read:3426 cache_create:1592 out:8
[→] Bash: Show working tree status

[+1s] [23:34:58] in:3 cache_read:3426 cache_create:1592 out:8
[→] Bash: Show unstaged changes

[+0s] [23:34:58] in:3 cache_read:3426 cache_create:1592 out:8
[→] Bash: Show recent commit messages

[+6s] [23:35:04] in:7 cache_read:5018 cache_create:3224 out:1
[→] Bash: Stage error handling changes and create commit

## Idea

maybe a deterministic shell script would help here? like use a fast haiku subagent or something to very quickly write a one line commit msg, and then run the script to add all changed files & commit them? This needs to be super fast.
