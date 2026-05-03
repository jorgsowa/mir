===description===
A variable not listed in a compact() call is not implicitly consumed by it
and is still reported as UnusedVariable.
===file===
<?php
function foo(): array {
    $name = 'Alice';
    $unlisted = 'ignored';
    return compact('name');
}
===expect===
UnusedVariable: Variable $unlisted is never read
===ignore===
TODO
