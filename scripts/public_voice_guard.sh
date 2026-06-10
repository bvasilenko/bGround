#!/usr/bin/env bash
set -euo pipefail

public_files() {
  find . \
    -path './.git' -prune -o \
    -path './target' -prune -o \
    -path './node_modules' -prune -o \
    -path './transcripts' -prune -o \
    -path './.github/docs/TODO.md' -prune -o \
    -path './contribot.config.*.json' -prune -o \
    -path './contribot.state.*.json' -prune -o \
    -type f \
    \( -name 'Cargo.toml' -o -name 'README.md' -o -name 'LICENSE' -o -name '*.rs' -o -name '*.yml' -o -name '*.yaml' -o -name '*.sh' \) \
    -print
}

scan_regex() {
  local pattern="$1"
  local message="$2"

  if public_files | xargs grep -nE "$pattern" >/tmp/bground_voice_guard_hits 2>/dev/null; then
    echo "$message" >&2
    cat /tmp/bground_voice_guard_hits >&2
    exit 1
  fi
}

scan_literal() {
  local pattern="$1"
  local message="$2"

  if public_files | xargs grep -nF "$pattern" >/tmp/bground_voice_guard_hits 2>/dev/null; then
    echo "$message" >&2
    cat /tmp/bground_voice_guard_hits >&2
    exit 1
  fi
}

scan_regex '(^|[^[:alpha:]])[Pp]ill[s]?([^[:alpha:]]|$)' 'public voice violation: internal package vocabulary found'
scan_regex 'Q5L R-[0-9]+' 'public voice violation: internal doctrine identifier found'
scan_literal "projects/b-suite"'/' 'public voice violation: internal workspace path found'
scan_literal "holding"'/' 'public voice violation: internal workspace path found'
scan_literal "frameworks"'/' 'public voice violation: internal workspace path found'
scan_regex 'B:[0-9]+' 'public voice violation: internal doctrine identifier found'
scan_regex 'I[0-9]+-I[0-9]+' 'public voice violation: internal invariant range found'
scan_literal "implementation-open"' gate' 'public voice violation: internal milestone vocabulary found'
scan_literal "0.1.0"'-skeleton' 'public voice violation: internal milestone vocabulary found'
scan_literal "PENDING"'-OPENEVOLVE-RUN' 'public voice violation: internal milestone vocabulary found'
scan_literal "PENDING"'-FIRST-CONTRIBOT-CYCLE' 'public voice violation: internal milestone vocabulary found'
scan_regex '[A-Z]+-[A-Z0-9-]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-[0-9]+' 'public voice violation: internal decision code found'
scan_literal "Co-Authored"'-By:' 'public voice violation: prohibited attribution found'
scan_literal "$(printf '\342\200\224')" 'public voice violation: em dash found'

expected='Prompt lookup tool. Agent names a claim type from a fixed list of 16; bground returns the prompt for that claim type. The prompt tells the agent how to check the claim against supplied evidence.'
if ! grep -Fq "description = \"$expected\"" crates/bground/Cargo.toml; then
  echo 'public voice violation: Cargo description does not match expected public text' >&2
  exit 1
fi

if ! grep -Fq "$expected" README.md; then
  echo 'public voice violation: README does not contain expected public text' >&2
  exit 1
fi
