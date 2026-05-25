===description===
unpackNonArrayKeyIterable
===file===
<?php
/** @suppress UnusedParam */
function foo(string ...$args): void {}

/** @var Iterator<float, string> */
$test = null;
foo(...$test);

===expect===
InvalidArgument
===ignore===
TODO
