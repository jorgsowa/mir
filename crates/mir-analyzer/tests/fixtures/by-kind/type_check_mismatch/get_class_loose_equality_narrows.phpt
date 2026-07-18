===description===
`get_class($x) == 'Foo'` / `!= 'Foo'` (loose comparison) narrows like the
strict `===`/`!==` form already does — class names are never
numeric-looking strings, so loose comparison agrees with strict here.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
final class Foo {}
final class Bar {}

function test_loose_equal(Foo|Bar $x): void {
    if (get_class($x) == 'Foo') {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

function test_reversed(Foo|Bar $x): void {
    if ('Foo' == get_class($x)) {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}

function test_loose_not_equal(Foo|Bar $x): void {
    if (get_class($x) != 'Foo') {
        /** @mir-check $x is Bar */
        $_ = $x;
    }
}

function test_dynamic_class_const_loose_equal(Foo|Bar $x): void {
    if ($x::class == 'Foo') {
        /** @mir-check $x is Foo */
        $_ = $x;
    }
}
===expect===
