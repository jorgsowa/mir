===description===
unpackNonArrayKeyIterable
===file===
<?php
/** @psalm-suppress UnusedParam */
function foo(string ...$args): void {}

/** @var Iterator<float, string> */
$test = null;
foo(...$test);

===expect===
InvalidArgument
===ignore===
TODO
