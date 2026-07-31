===description===
Negative control for the J1 refined-string-scalar-subtype fix: a genuinely
non-scalar value (array) must still be rejected by a `scalar`-typed param.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php

/** @param scalar $v */
function acceptsScalar($v): void {}

function f(array $arr): void {
    acceptsScalar($arr);
}
===expect===
InvalidArgument@7:18-7:22: Argument $v of acceptsScalar() expects 'scalar', got 'array'
