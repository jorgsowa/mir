===description===
(int) cast on a pure array type still emits InvalidCast — no scalar-safe atoms present
===config===
suppress=UnusedVariable
===file===
<?php
function getArray(): array {
    return [];
}

$x = (int) getArray();
===expect===
InvalidCast@6:11-6:21: Cannot cast 'array<mixed, mixed>' to 'int'
