===description===
enumStringOrEnumIntIncorrectInt
===file===
<?php
namespace Ns;

/** @psalm-param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidArgument
===ignore===
TODO
