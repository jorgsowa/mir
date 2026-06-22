===description===
is_subclass_of() uses strict-subclass semantics: the exact class is NOT a
subclass of itself. True branch keeps known subclasses; false branch must NOT
remove the exact class (doing so would wrongly mark Foo uses as diverging).
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Cat extends Animal {}

function needs_int(int $i): void {}

/** @param Animal|Dog|null $obj */
function test_true_branch_keeps_subclass(mixed $obj): void {
    if (is_subclass_of($obj, 'Animal')) {
        // Dog is a subclass of Animal and should remain in the true branch.
        /** @mir-check $obj is Dog */
        $_ = $obj;
    }
}

/** @param Animal $obj */
function test_false_branch_stays_alive(Animal $obj): void {
    // Animal is NOT a strict subclass of itself, so is_subclass_of($obj, 'Animal')
    // can return false even when $obj is Animal. The false branch must NOT diverge —
    // before the fix, filter_out_instanceof_match removed Animal from Animal (empty),
    // then mark_diverges=true caused ctx.diverges, suppressing needs_int below.
    // After the fix: false branch does no narrowing → branch alive → InvalidArgument fires.
    if (!is_subclass_of($obj, 'Animal')) {
        needs_int($obj);
    }
}

/** @param Animal|Dog|string $obj */
function test_true_branch_drops_exact_and_non_object(mixed $obj): void {
    if (is_subclass_of($obj, 'Animal')) {
        // After strict-subclass narrowing on Animal|Dog|string:
        //   Animal dropped (not a strict subclass of itself)
        //   Dog kept (strict subclass)
        //   string dropped (not an object)
        // Only Dog survives.
        /** @mir-check $obj is Dog */
        $_ = $obj;
    }
}
===expect===
InvalidArgument@25:18-25:22: Argument $i of needs_int() expects 'int', got 'Animal'
