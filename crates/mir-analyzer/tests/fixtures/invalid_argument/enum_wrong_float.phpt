===description===
enumWrongFloat
===file===
<?php
namespace Ns;

/** @psalm-param 1.2|3.4|5.6 $s */
function foo($s) : void {}
foo(7.8);
===expect===
InvalidArgument
===ignore===
TODO
