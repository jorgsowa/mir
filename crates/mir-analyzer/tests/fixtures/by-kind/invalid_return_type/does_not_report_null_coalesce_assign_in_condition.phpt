===description===
does not report null coalesce assign in condition
===file===
<?php
function f(): object|null { return null; }

// ??= in if condition: after !($s ??= f()) exits, $s must be object (non-null truthy)
function a(object|null $s): object {
    if (!($s ??= f())) { exit; }
    return $s;
}
===expect===
