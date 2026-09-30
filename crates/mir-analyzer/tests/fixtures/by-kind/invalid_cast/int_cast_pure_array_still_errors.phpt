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
//         ^^^^^^^^^^ InvalidCast: Cannot cast 'array' to 'int'
===expect===
