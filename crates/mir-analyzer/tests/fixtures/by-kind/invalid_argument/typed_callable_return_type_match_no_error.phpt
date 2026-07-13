===description===
Sibling of typed_callable_return_type_mismatch: a matching return type
must stay silent.
===file===
<?php
/**
 * @param callable(int):string $cb
 */
function takesCb(callable $cb): void {
    echo $cb(1);
}

function giveString(int $x): string {
    return (string) $x;
}

takesCb('giveString');
===expect===
