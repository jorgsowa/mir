===description===
is_a() narrows using instanceof semantics (includes the exact class).
is_subclass_of() uses strict-subclass semantics (the exact class is excluded).
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php
class Foo {}
class Bar {}
class Baz extends Foo {}

/** @param Foo|Bar|null $obj */
function test_is_a_class_const(mixed $obj): void {
    if (is_a($obj, Foo::class)) {
        /** @mir-check $obj is Foo */
        $_ = $obj;
    }
}

/** @param Foo|Bar|null $obj */
function test_is_a_string(mixed $obj): void {
    if (is_a($obj, 'Foo')) {
        /** @mir-check $obj is Foo */
        $_ = $obj;
    }
}

/** @param Foo|Bar|Baz|null $obj */
function test_is_subclass_of(mixed $obj): void {
    if (is_subclass_of($obj, 'Foo')) {
        // is_subclass_of is a strict check: Foo itself is NOT a subclass of Foo,
        // only Baz (which extends Foo) survives in the true branch.
        /** @mir-check $obj is Baz */
        $_ = $obj;
    }
}
===expect===
