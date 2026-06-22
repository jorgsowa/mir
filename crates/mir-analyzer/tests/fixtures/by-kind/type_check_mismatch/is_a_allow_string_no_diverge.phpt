===description===
is_a($x, 'Foo', true) with $allow_string=true must not narrow a string/class-string
variable to an object type, and must not mark the true branch as diverging.
String and class-string atoms are valid is_a()-true values and must be preserved.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}
class Bar {}

/** @param class-string $cls */
function test_class_string_preserved(string $cls): void {
    if (is_a($cls, 'Foo', true)) {
        // $cls is a class-string; the true branch must NOT narrow it away to
        // an object type — it should remain a class-string type.
        /** @mir-check $cls is class-string */
        $_ = $cls;
    }
}

function test_string_preserved(string $name): void {
    if (is_a($name, 'Foo', true)) {
        // Plain string: may be a class name. Must not be erased to an object
        // type, and the branch must remain reachable.
        /** @mir-check $name is string */
        $_ = $name;
    }
}

/** @param Foo|Bar $obj */
function test_object_branch_still_works(mixed $obj): void {
    // For object types, is_a with allow_string still narrows correctly.
    if (is_a($obj, 'Foo', true)) {
        /** @mir-check $obj is Foo */
        $_ = $obj;
    }
}

/** @param Foo $obj */
function test_false_branch_no_false_diverge(Foo $obj): void {
    // With allow_string, even an exact-class type must not diverge on false
    // branch (allow_string semantics extend reachability).
    if (!is_a($obj, 'Foo', true)) {
        echo "unreachable but no diverge analysis hole";
    }
}
===expect===
