===description===
Enum wrong float
===file===
<?php
namespace Ns;

/** @param 1.2|3.4|5.6 $s */
function foo($s) : void {}
foo(7.8);
===expect===
InvalidArgument
===ignore===
TODO
