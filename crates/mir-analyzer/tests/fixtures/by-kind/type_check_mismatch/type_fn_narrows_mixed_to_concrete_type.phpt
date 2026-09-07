===description===
is_int()/is_string()/is_float()/is_bool()/is_null()/is_array()/is_scalar() guards
must narrow a `mixed`-typed value to the concrete checked type, not leave it as
`mixed` — narrow_to_*() previously kept TMixed atoms unchanged via a plain
filter() instead of substituting the narrowed concrete type.
===file===
<?php

function checkInt(mixed $x): void {
    if (is_int($x)) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function checkString(mixed $x): void {
    if (is_string($x)) {
        /** @mir-check $x is string */
        echo $x;
    }
}

function checkFloat(mixed $x): void {
    if (is_float($x)) {
        /** @mir-check $x is float */
        echo $x;
    }
}

function checkBool(mixed $x): void {
    if (is_bool($x)) {
        /** @mir-check $x is bool */
        echo $x ? "y" : "n";
    }
}

function checkNull(mixed $x): void {
    if (is_null($x)) {
        /** @mir-check $x is null */
        echo "null";
    }
}

function checkArray(mixed $x): void {
    if (is_array($x)) {
        /** @mir-check $x is array<mixed, mixed> */
        echo count($x);
    }
}

function checkScalar(mixed $x): void {
    if (is_scalar($x)) {
        /** @mir-check $x is scalar */
        echo "";
    }
}
===expect===
