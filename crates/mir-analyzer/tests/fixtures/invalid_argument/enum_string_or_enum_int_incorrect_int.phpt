===description===
enumStringOrEnumIntIncorrectInt
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidArgument
===ignore===
TODO
