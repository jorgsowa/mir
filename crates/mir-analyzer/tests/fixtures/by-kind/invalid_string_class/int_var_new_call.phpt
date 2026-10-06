===description===
Int var new call
===file===
<?php
$a = 5;
new $a();
//  ^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got '5'
