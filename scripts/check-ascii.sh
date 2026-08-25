#!/bin/sh
# Every tracked text file must be pure ASCII.
#
# The repository was deliberately de-typographied: em dashes, en dashes,
# ellipses, arrows, the multiplication and micro signs all had ASCII spellings
# that read as well, and mixing the two is worse than either. This keeps them
# out. Write `-` or a comma for a dash, `...`, `->`, `x`, `us`.
#
# It matters beyond taste in one place: the diagnostic text in `traits.rs` is
# pinned character for character in `tests/ui/*.stderr`.
#
#   scripts/check-ascii.sh              # every tracked file
#   scripts/check-ascii.sh FILE...      # just these
#
# `LC_ALL=C` is what makes this work: grep then matches bytes rather than
# characters, so every byte of a UTF-8 sequence falls outside printable ASCII
# and the negated class catches it. `-I` skips binary files, and `/dev/null`
# is a second file so grep prints the name even when given exactly one.
set -eu
cd "$(dirname "$0")/.."

if [ "$#" -eq 0 ]; then
    IFS='
'
    set -- $(git ls-files)
    [ "$#" -gt 0 ] || exit 0
fi

tab=$(printf '\t')
found=$(LC_ALL=C grep -n -I "[^${tab} -~]" "$@" /dev/null || true)

if [ -n "$found" ]; then
    printf '%s\n' "$found" >&2
    printf '\nNon-ASCII above. This repository is ASCII only; see the header of %s.\n' \
        "scripts/check-ascii.sh" >&2
    exit 1
fi
