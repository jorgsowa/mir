===description===
Int var new call
===file===
<?php
$a = 5;
new $a();
===expect===
InvalidStringClass@3:5-3:7: Dynamic class instantiation requires string or class-string type, got '5'
