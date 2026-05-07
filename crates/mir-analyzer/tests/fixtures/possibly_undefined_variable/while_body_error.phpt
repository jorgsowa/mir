===description===
while body error
===file===
<?php
function foo(bool $c): int {
    while ($c) {
        $x = 1;
        $c = false;
    }
    return $x;
}
===expect===
PossiblyUndefinedVariable@7:11: Variable $x might not be defined
