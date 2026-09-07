===description===
FP-L19: an assignment made in an earlier elseif's own CONDITION was lost on
the false edge — each elseif re-branched from the pre-if context and only
ever re-narrowed the PRIMARY if's condition, discarding every earlier
elseif's condition (both its assignment and its narrowing) entirely.
===config===
suppress=UnusedVariable,MixedAssignment,UnusedParam
===file===
<?php

function tryA(): ?string {
    return null;
}

function tryB(): ?string {
    return null;
}

function use1(string $s): void {}

function f(): void {
    if ($a = tryA()) {
        use1($a);
    } elseif ($b = tryB()) {
        use1($b);
    } elseif ($a || $b) {
        // Reaching here means both prior conditions were falsy — $a and $b
        // were still both assigned (to null) by their own condition.
        use1((string) ($a ?? $b));
    }
}
===expect===
