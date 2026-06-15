===description===
Enum string or enum int incorrect string
===config===
suppress=UnusedParam
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo("bat");
===expect===
InvalidArgument@6:4-6:9: Argument $s of foo() expects '"foo"|"bar"|1|2|3', got '"bat"'
