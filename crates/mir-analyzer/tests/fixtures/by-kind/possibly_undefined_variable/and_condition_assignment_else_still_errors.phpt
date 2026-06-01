===description===
PossiblyUndefinedVariable still fires when var assigned in && condition is used in the else branch
===file===
<?php
function foo(bool $a, object $obj): void {
    if ($a && ! is_null($y = $obj->getY())) {
        echo $y; // ok: $y definitely assigned in true-branch
    } else {
        echo $y; // error: $a might have been false, so $y was never assigned
    }
}
===expect===
PossiblyUndefinedVariable@6:14-6:16: Variable $y might not be defined
