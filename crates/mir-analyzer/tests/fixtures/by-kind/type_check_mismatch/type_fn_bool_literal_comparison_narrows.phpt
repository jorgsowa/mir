===description===
`is_string($x) === true` / `== true` / `=== false` / `== false` (and
`$x instanceof Y` compared the same way) never narrowed $x — the bool-
literal comparison arms only tried extract_var_name/extract_prop_access
on the OTHER operand, never recursing into narrow_from_condition to
dispatch the FunctionCall/Instanceof arm a bare `if (is_string($x))`
already gets.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function identicalTrue(int|string $x): void {
    if (is_string($x) === true) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

function identicalFalse(int|string $x): void {
    if (is_string($x) === false) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

function looseTrue(int|string $x): void {
    if (is_string($x) == true) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

function looseFalse(int|string $x): void {
    if (is_string($x) == false) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

function reversedOperandOrder(int|string $x): void {
    if (true === is_int($x)) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

class Foo {}
class Bar {}
function instanceofIdenticalTrue(Foo|Bar $o): void {
    if (($o instanceof Foo) === true) {
        /** @mir-check $o is Foo */
        $_ = $o;
    }
}
===expect===
