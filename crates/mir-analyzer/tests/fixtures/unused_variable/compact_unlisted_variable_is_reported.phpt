===file===
<?php
function foo(): array {
    $name = 'Alice';
    $unlisted = 'ignored';
    return compact('name');
}
===expect===
UnusedVariable: Variable $unlisted is never read
