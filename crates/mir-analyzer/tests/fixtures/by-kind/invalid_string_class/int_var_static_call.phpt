===description===
Int var static call
===file===
<?php
$a = 5;
$a::bar();
===expect===
InvalidStringClass@3:0-3:2: Dynamic class instantiation requires string or class-string type, got '5'
