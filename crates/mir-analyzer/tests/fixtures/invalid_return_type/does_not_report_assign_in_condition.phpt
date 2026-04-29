===file===
<?php
function f(): object|null { return null; }

// regular = assignment in if condition: after !($s = f()) exits, $s must be object
function d(object|null $s): object {
    if (!($s = f())) { exit; }
    return $s;
}
===expect===
