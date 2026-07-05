===description===
A closure containing `yield` is itself a generator function — calling it
returns a Generator, same as a top-level function with no declared return
type. The closure gets its own StatementsAnalyzer, so its yields must be
read from its own `yielded_types`, not silently dropped in favor of only
`return_types`.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
$gen = function () {
    yield 1;
    yield 2;
};
$g = $gen();
/** @mir-check $g is Generator<int, 1|2, mixed, void> */
$_ = 1;
===expect===
