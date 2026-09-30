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
//  ^^^^^ InvalidArgument: Argument $s of foo() expects '"foo"|"bar"|1|2|3', got '"bat"'
===expect===
