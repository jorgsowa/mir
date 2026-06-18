===description===
Truthy check on `bool` narrows to `true`; falsy check narrows to `false`.
Both branches of `if ($boolVar)` should produce the literal type, not the wide bool.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param bool $x */
function test_bool_if(bool $x): void {
    if ($x) {
        /** @mir-check $x is true */
        $_ = $x;
    } else {
        /** @mir-check $x is false */
        $_ = $x;
    }
}

/** @param bool|null $y */
function test_nullable_bool(bool|null $y): void {
    if ($y) {
        /** @mir-check $y is true */
        $_ = $y;
    } else {
        /** @mir-check $y is false|null */
        $_ = $y;
    }
}
===expect===
