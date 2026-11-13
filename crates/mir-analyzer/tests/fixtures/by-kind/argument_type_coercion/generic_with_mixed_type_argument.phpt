===description===
A generic object whose type argument is unknown (`mixed`) may hold the expected one, so
passing it is a coercion, not an invalid argument.
===file===
<?php
/** @param ArrayIterator<int, string> $t */
function p(ArrayIterator $t): void { echo count($t); }

/** @return array */
function doc(): array { return []; }

function f(): void {
    p(new ArrayIterator(doc()));
//    ^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $t of p() expects 'ArrayIterator<int, string>', got 'ArrayIterator<int|string, mixed>' — coercion may fail at runtime
}
