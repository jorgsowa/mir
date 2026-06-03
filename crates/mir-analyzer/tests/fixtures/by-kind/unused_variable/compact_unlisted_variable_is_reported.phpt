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
UnusedVariable@3:5-3:10: Variable $name is never read
UnusedVariable@4:5-4:14: Variable $unlisted is never read
