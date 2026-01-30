# Compact version - only shows assistant messages and tool names
# Usage: cat output.json | jq -f parse-claude-compact.jq -r

select(.type != "system") |

if .type == "assistant" and (.message.content // null) then
  # Show usage info first if available (with timestamp)
  (if .message.usage then
    (now | strftime("%H:%M:%S")) as $timestamp |
    "[\u001b[90m\($timestamp)\u001b[0m] in:\(.message.usage.input_tokens // 0) cache_read:\(.message.usage.cache_read_input_tokens // 0) cache_create:\(.message.usage.cache_creation_input_tokens // 0) out:\(.message.usage.output_tokens // 0)"
  else
    empty
  end),
  # Then show content
  (.message.content[] |
    if .type == "text" then
      "[\u001b[1;36mASSISTANT\u001b[0m] \(.text)"
    elif .type == "tool_use" then
      if .name == "Bash" then
        "[\u001b[1;33m→\u001b[0m] \(.name): \(.input.description // .input.command[0:60])"
      elif .name == "Task" then
        "[\u001b[1;35m⚡\u001b[0m] Subagent: \(.input.subagent_type) - \(.input.description)"
      elif .name == "Write" then
        "[\u001b[1;33m→\u001b[0m] \(.name): \(.input.file_path)"
      elif .name == "Edit" then
        "[\u001b[1;33m→\u001b[0m] \(.name): \(.input.file_path)"
      elif .name == "Read" then
        "[\u001b[1;33m→\u001b[0m] \(.name): \(.input.file_path)"
      elif .name == "Grep" then
        "[\u001b[1;33m→\u001b[0m] \(.name): pattern=\(.input.pattern)\(if .input.path then " path=\(.input.path)" else "" end)\(if .input.output_mode then " mode=\(.input.output_mode)" else "" end)\(if .input.glob then " glob=\(.input.glob)" else "" end)"
      elif .name == "Glob" then
        "[\u001b[1;33m→\u001b[0m] \(.name): pattern=\(.input.pattern)\(if .input.path then " path=\(.input.path)" else "" end)"
      elif .name == "TodoWrite" then
        if (.input.todos | type) == "array" then
          "[\u001b[1;33m→\u001b[0m] \(.name): \(.input.todos | length) todos (\([.input.todos[] | .status] | group_by(.) | map("\(.[0]):\(length)") | join(", ")))"
        else
          "[\u001b[1;33m→\u001b[0m] \(.name)"
        end
      elif .name == "Skill" then
        if .input.args then
          "[\u001b[1;35m✦\u001b[0m] Skill: \(.input.skill) \(.input.args)"
        else
          "[\u001b[1;35m✦\u001b[0m] Skill: \(.input.skill)"
        end
      else
        "[\u001b[1;33m→\u001b[0m] \(.name)"
      end
    else
      empty
    end
  ),
  # Add blank line after each assistant message
  ""

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
