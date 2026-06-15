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
PossiblyUndefinedVariable@8:11-8:13: Variable $x might not be defined
