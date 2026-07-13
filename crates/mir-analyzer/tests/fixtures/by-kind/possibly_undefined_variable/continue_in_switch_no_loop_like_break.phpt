===description===
`continue;` inside a switch with no enclosing loop behaves like `break`
(PHP semantics: switch counts as one loop level) — only possibly-undefined,
not a hard undefined-variable error.
===file===
<?php
function foo(int $i): void {
    switch ($i) {
        case 0:
            if (rand(0, 1)) {
                continue;
            }

        default:
            $a = true;
    }

    if ($a) {}
}
===expect===
PossiblyUndefinedVariable@13:8-13:10: Variable $a might not be defined
