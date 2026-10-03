===description===
N5 (false-branch fix): TCallableString must be recognised as callable so the
!is_callable false branch correctly removes it. A callable-string atom is
definitionally callable, so is_callable() is always true for it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function needs_int(int $i): void {}
function needs_string(string $s): void {}

/** @param callable-string $fn */
function test_not_callable_of_callable_string_is_redundant(mixed $fn): void {
    if (!is_callable($fn)) {}
//      ^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

/** @param callable-string $fn */
function test_callable_of_callable_string_is_redundant(mixed $fn): void {
    if (is_callable($fn)) {
//      ^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        needs_int($fn);
//                ^^^ InvalidArgument: Argument $i of needs_int() expects 'int', got 'callable-string'
    }
}

/** @param callable-string|int $x */
function test_union_true_branch_narrows_out_int(mixed $x): void {
    if (is_callable($x)) {
        needs_int($x);
//                ^^ InvalidArgument: Argument $i of needs_int() expects 'int', got 'callable-string'
    }
}

/** @param callable-string|int $x */
function test_union_false_branch_narrows_out_callable_string(mixed $x): void {
    if (!is_callable($x)) {
        needs_string($x);
//                   ^^ ArgumentTypeCoercion: Argument $s of needs_string() expects 'string', got 'int' — coercion may fail at runtime
    }
}
===expect===
