===description===
Enum string or enum int incorrect string
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo("bat");
===expect===
InvalidArgument
===ignore===
TODO
