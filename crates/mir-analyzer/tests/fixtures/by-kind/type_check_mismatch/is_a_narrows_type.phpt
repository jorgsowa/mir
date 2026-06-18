===description===
is_a() and is_subclass_of() narrow the first argument to the named class in the true branch.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php
class Foo {}
class Bar {}

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

/** @param Foo|Bar|null $obj */
function test_is_subclass_of(mixed $obj): void {
    if (is_subclass_of($obj, 'Foo')) {
        /** @mir-check $obj is Foo */
        $_ = $obj;
    }
}
===expect===
