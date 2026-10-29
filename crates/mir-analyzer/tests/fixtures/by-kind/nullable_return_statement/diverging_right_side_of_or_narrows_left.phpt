===description===
`A or die()` and `A || throw ...` continue only where `A` held, and `A and die()` only where it
didn't; a right side that returns keeps the left side's type unnarrowed.
===file===
<?php
function orDie(?string $s): string {
    is_string($s) or die();
    return $s;
}

function orThrow(?string $s): string {
    $s !== null || throw new RuntimeException();
    return $s;
}

function andExit(?string $s): string {
    $s === null and exit(1);
    return $s;
}

function orPrint(?string $s): string {
    is_string($s) or print('missing');
    return $s;
//  ^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
}
