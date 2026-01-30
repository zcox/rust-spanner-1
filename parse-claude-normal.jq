# Parse Claude CLI JSON output to show important information
# Usage: cat output.json | jq -f parse-claude-output.jq -r

# Skip system init messages
select(.type != "system") |

# Process different message types
if .type == "assistant" and (.message.content // null) then
  # Show usage info first if available
  (if .message.usage then
    "[\u001b[90mTokens\u001b[0m] in:\(.message.usage.input_tokens // 0) cache_read:\(.message.usage.cache_read_input_tokens // 0) cache_create:\(.message.usage.cache_creation_input_tokens // 0) out:\(.message.usage.output_tokens // 0)"
  else
    empty
  end),
  # Then show content
  (.message.content[] |
    # Extract text messages
    if .type == "text" then
      "[\u001b[1;36mASSISTANT\u001b[0m] \(.text)"

    # Extract tool uses
    elif .type == "tool_use" then
      if .name == "Bash" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.description // "no description")\n  Command: \(.input.command)"
      elif .name == "Task" then
        "[\u001b[1;35mSUBAGENT\u001b[0m] \(.input.subagent_type): \(.input.description)\n  Prompt: \(.input.prompt | split("\n")[0] + (if (.input.prompt | split("\n") | length) > 1 then "..." else "" end))"
      elif .name == "Read" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.file_path | split("/")[-1])"
      elif .name == "Write" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.file_path | split("/")[-1]) (\(.input.content | length) bytes)"
      elif .name == "Edit" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.file_path | split("/")[-1])"
      elif .name == "Glob" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.pattern)"
      elif .name == "Grep" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.pattern)\(.input.path // "")"
      elif .name == "TodoWrite" then
        "[\u001b[1;33mTOOL\u001b[0m] \(.name): \(.input.todos | length) todos"
      elif .name == "Skill" then
        if .input.args then
          "[\u001b[1;35mSKILL\u001b[0m] \(.input.skill) \(.input.args)"
        else
          "[\u001b[1;35mSKILL\u001b[0m] \(.input.skill)"
        end
      else
        "[\u001b[1;33mTOOL\u001b[0m] \(.name)"
      end
    else
      empty
    end
  ),
  # Add blank line after each assistant message
  ""

# Extract tool results (condensed)
elif .type == "user" and (.message.content // null) then
  .message.content[] |
  if .type == "tool_result" then
    # Show truncated content for tool results
    if .content then
      if (.content | type) == "string" then
        if (.content | length) > 200 then
          "[\u001b[1;32mRESULT\u001b[0m] \(.content[0:200])...\n"
        else
          "[\u001b[1;32mRESULT\u001b[0m] \(.content)\n"
        end
      else
        "[\u001b[1;32mRESULT\u001b[0m] \(.content | tostring | .[0:200])...\n"
      end
    else
      empty
    end
  else
    empty
  end

# Show final summary for result type
elif .type == "result" then
  "",
  "═══════════════════════════════════════════════════════════",
  "[\u001b[1;32mFINAL SUMMARY\u001b[0m]",
  "Duration: \(.duration_ms / 1000)s (\(.num_turns) turns)",
  "Total Cost: $\(.total_cost_usd)",
  "",
  "Token Usage:",
  "  Input: \(.usage.input_tokens // 0) tokens",
  "  Cache Read: \(.usage.cache_read_input_tokens // 0) tokens",
  "  Cache Created: \(.usage.cache_creation_input_tokens // 0) tokens",
  "  Output: \(.usage.output_tokens // 0) tokens",
  (if .modelUsage then
    "",
    "Model Breakdown:",
    (.modelUsage | to_entries[] |
      "  \(.key):",
      "    Input: \(.value.inputTokens) | Cache Read: \(.value.cacheReadInputTokens) | Cache Created: \(.value.cacheCreationInputTokens) | Output: \(.value.outputTokens)",
      "    Cost: $\(.value.costUSD)"
    )
  else
    empty
  end),
  "═══════════════════════════════════════════════════════════"
else
  empty
end
