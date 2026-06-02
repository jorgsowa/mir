===description===
Enum string or enum int incorrect string
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo("bat");
===expect===
InvalidArgument@6:5-6:10: Argument $s of foo() expects '"foo"|"bar"|1|2|3', got '"bat"'
