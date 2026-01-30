#!/usr/bin/env bash
# Show elapsed time between turns to find slow parts with color coding
# Usage: ./show-turn-timing.sh [file]
# If no file provided, reads from stdin

if [ -n "$1" ]; then
    cat "$1"
else
    cat
fi | perl -ne '
BEGIN {
    $| = 1;  # Autoflush STDOUT for real-time streaming
    $prev_time = 0;

    # Timing buckets (easy to customize)
    # Format: [threshold_seconds, color_code]
    @buckets = (
        [2,  "\033[32m"],      # 0-2s:   green (fast)
        [5,  "\033[33m"],      # 3-5s:   yellow (medium)
        [10, "\033[38;5;208m"], # 6-10s:  orange (slow)
        [20, "\033[31m"],      # 11-20s: red (very slow)
        [999, "\033[1;31m"]    # 21+s:   bold red (extremely slow)
    );
    $reset = "\033[0m";
}

sub get_color {
    my ($elapsed) = @_;
    for my $bucket (@buckets) {
        if ($elapsed <= $bucket->[0]) {
            return $bucket->[1];
        }
    }
    return $buckets[-1][1];  # Default to last color
}

if (/\[.*?(\d{2}):(\d{2}):(\d{2}).*?\]/) {
    $curr_time = $1 * 3600 + $2 * 60 + $3;

    if ($prev_time > 0) {
        $elapsed = $curr_time - $prev_time;
        $elapsed += 86400 if $elapsed < 0;  # Handle day rollover
        $color = get_color($elapsed);
        print "${color}[+${elapsed}s]${reset} $_";
    } else {
        print $_;
    }
    $prev_time = $curr_time;
} else {
    print $_;
}
'
