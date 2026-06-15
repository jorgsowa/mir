===description===
if only
===file===
<?php
function foo(bool $c): string {
    if ($c) { $r = 'hello'; }
    return $r;
}
===expect===
PossiblyUndefinedVariable@4:11-4:13: Variable $r might not be defined
