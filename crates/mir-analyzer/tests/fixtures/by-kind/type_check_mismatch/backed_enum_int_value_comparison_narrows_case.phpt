===description===
The `->value` comparison narrowing also applies to an int-backed enum, not
just string-backed — the literal side is an int, not a string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
enum Priority: int {
    case Low = 1;
    case Medium = 2;
    case High = 3;
}

function trueBranchNarrows(Priority $p): void {
    if ($p->value === 2) {
        /** @mir-check $p is Priority::Medium */
        $_ = 1;
    }
}

function falseBranchExcludes(Priority $p): void {
    if ($p->value === 2) {
        return;
    }
    /** @mir-check $p is Priority::Low|Priority::High */
    $_ = 1;
}
===expect===
