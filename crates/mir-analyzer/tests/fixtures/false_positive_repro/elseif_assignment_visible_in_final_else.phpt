===description===
FP-L19 (else-branch variant): an assignment made in an elseif's own
CONDITION was lost in the final `else` branch too — the old else_ctx was a
separate branch off the pre-if context that only ever type-narrowed
elseif conditions, never analyzed their expressions, so an assignment made
in an elseif condition never became visible in `else`.
===config===
suppress=UnusedVariable,MixedAssignment,UnusedParam
===file===
<?php

function tryA(): ?string {
    return null;
}

function cond(): bool {
    return false;
}

function use1(string $s): void {}

function f(): void {
    if (cond()) {
    } elseif ($a = tryA()) {
        use1($a);
    } else {
        use1((string) $a);
    }
}
===expect===
