===description===
no PossiblyUndefinedVariable when ?? is the safe fallback
===file===
<?php
function foo(bool $c): string {
    if ($c) { $r = 'hello'; }
    return $r ?? 'default';
}
===expect===
