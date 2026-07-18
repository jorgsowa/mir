===description===
`$obj::class` (PHP 8 shorthand for get_class($obj)) narrows like get_class().
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
final class Foo {}
final class Bar {}

function test_literal(Foo|Bar $x): void {
    if ($x::class === Foo::class) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

function test_literal_string(Foo|Bar $x): void {
    if ($x::class === 'Foo') {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

function test_reversed(Foo|Bar $x): void {
    if (Foo::class === $x::class) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

function test_not_equal(Foo|Bar $x): void {
    if ($x::class !== Foo::class) {
        /** @mir-check $x is Bar */
        $_ = $x;
    }
}
===expect===
