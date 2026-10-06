===description===
if only
===file===
<?php
function foo(bool $c): string {
    if ($c) { $r = 'hello'; }
    return $r;
//         ^^ PossiblyUndefinedVariable: Variable $r might not be defined
}
