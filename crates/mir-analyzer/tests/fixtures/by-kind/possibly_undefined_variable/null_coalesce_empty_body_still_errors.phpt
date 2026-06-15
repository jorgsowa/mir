===description===
PossiblyUndefinedVariable still flagged after ?? guard with empty if-body
===file===
<?php
function foo(bool $c): string {
    if ($c) { $x = 'hello'; }
    if (($x ?? false) === false) { /* empty - no assignment */ }
    return $x;
}
===expect===
PossiblyUndefinedVariable@5:11-5:13: Variable $x might not be defined
