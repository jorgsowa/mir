===description===
intVarStaticCall
===file===
<?php
$a = 5;
$a::bar();
===expect===
InvalidStringClass@3:1: Dynamic class instantiation requires string or class-string type, got '5'
