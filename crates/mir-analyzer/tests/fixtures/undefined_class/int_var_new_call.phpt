===description===
intVarNewCall
===file===
<?php
$a = 5;
new $a();
===expect===
InvalidStringClass@3:5: Dynamic class instantiation requires string or class-string type, got '5'
