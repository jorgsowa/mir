===description===
is_a($x, 'Foo', true) with a class-string<X> atom checks X against Foo:
an unrelated class-string is dropped in the true branch and dropped from
the false branch only when it's a provable match, mirroring the object
side of the same check.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}
class Bar {}
class Baz extends Foo {}

/** @param class-string<Foo>|class-string<Bar> $cls */
function test_true_branch_drops_unrelated_class_string(string $cls): void {
    if (is_a($cls, 'Foo', true)) {
        /** @mir-check $cls is class-string<Foo> */
        $_ = $cls;
    }
}

/** @param class-string<Baz>|class-string<Bar> $cls */
function test_true_branch_keeps_subclass_string(string $cls): void {
    if (is_a($cls, 'Foo', true)) {
        // Baz extends Foo, so is_a() includes it (instanceof semantics).
        /** @mir-check $cls is class-string<Baz> */
        $_ = $cls;
    }
}

/** @param class-string<Foo>|class-string<Bar> $cls */
function test_false_branch_drops_matching_class_string(string $cls): void {
    if (is_a($cls, 'Foo', true)) {
        // handled above
    } else {
        /** @mir-check $cls is class-string<Bar> */
        $_ = $cls;
    }
}

/** @param class-string $cls */
function test_generic_class_string_still_preserved(string $cls): void {
    // No specific name to check — must not be erased or narrowed further.
    if (is_a($cls, 'Foo', true)) {
        /** @mir-check $cls is class-string */
        $_ = $cls;
    }
}

/** @param string $name */
function test_generic_string_still_preserved(string $name): void {
    if (is_a($name, 'Foo', true)) {
        /** @mir-check $name is string */
        $_ = $name;
    }
}
===expect===
