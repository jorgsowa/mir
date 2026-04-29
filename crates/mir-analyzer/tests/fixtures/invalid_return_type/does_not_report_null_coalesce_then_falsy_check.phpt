===file===
<?php
function f(): object|null { return null; }

// ??= as a statement then if (!$s): baseline form that should already pass
function c(object|null $s): object {
    $s ??= f();
    if (!$s) { exit; }
    return $s;
}
===expect===
