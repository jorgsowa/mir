===description===
check_typed_callable_arg checked parameter contravariance but never the
return type — a callable(int):string accepting a callable returning int
went unflagged.
===file===
<?php
/**
 * @param callable(int):string $cb
 */
function takesCb(callable $cb): void {
    echo $cb(1);
}

function giveInt(int $x): int {
    return $x + 1;
}

takesCb('giveInt');
===expect===
InvalidArgument@13:8-13:17: Argument $cb of takesCb() expects 'callable returning string', got 'callable returning int'
