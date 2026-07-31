===description===
Negative control for the L19 elseif-chain fix: a variable never assigned
in any preceding condition must still be flagged undefined in a later
elseif/else — the fix only carries forward REAL assignments, it doesn't
suppress checking altogether.
===config===
suppress=UnusedParam,MixedArgument
===file===
<?php

function tryA(): ?string {
    return null;
}

function use1(string $s): void {}

function f(): void {
    if ($a = tryA()) {
        use1($a);
    } elseif ($a === null) {
        use1($never_assigned);
    }
}
===expect===
UndefinedVariable@13:13-13:28: Variable $never_assigned is not defined
