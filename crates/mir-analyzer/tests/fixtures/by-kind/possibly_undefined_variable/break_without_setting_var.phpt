===description===
Break without setting var
===file===
<?php
function foo(int $i) : void {
    switch ($i) {
        case 0:
            if (rand(0, 1)) {
                break;
            }

        default:
            $a = true;
    }

    if ($a) {}
//      ^^ PossiblyUndefinedVariable: Variable $a might not be defined
}
