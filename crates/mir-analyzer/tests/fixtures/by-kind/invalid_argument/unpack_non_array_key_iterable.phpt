===description===
Unpacking an `Iterator<float, string>` (a non-int key type) now resolves
its real value type (`string`) via the interface's own type args instead
of bailing to `mixed` on a naive int-key-only heuristic — G4.
===file===
<?php
/** @suppress UnusedParam */
function foo(string ...$args): void {}

/** @var Iterator<float, string> */
$test = null;
foo(...$test);

===expect===
