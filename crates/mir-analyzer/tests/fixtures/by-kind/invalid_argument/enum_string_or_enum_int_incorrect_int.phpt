===description===
Enum string or enum int incorrect int
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidArgument@6:5-6:6: Argument $s of foo() expects '"foo"|"bar"|1|2|3', got '4'
