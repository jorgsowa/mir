===description===
nested if error
===file===
<?php
function foo(bool $a, bool $b): int {
    if ($a) {
        if ($b) {
            $x = 1;
        }
    }
    return $x;
}
===expect===
PossiblyUndefinedVariable@8:12: Variable $x might not be defined
