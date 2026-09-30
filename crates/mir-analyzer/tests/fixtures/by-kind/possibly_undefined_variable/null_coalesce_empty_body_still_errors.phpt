===description===
PossiblyUndefinedVariable still flagged after ?? guard with empty if-body
===file===
<?php
function foo(bool $c): string {
    if ($c) { $x = 'hello'; }
    if (($x ?? false) === false) { /* empty - no assignment */ }
    return $x;
//         ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
===expect===
