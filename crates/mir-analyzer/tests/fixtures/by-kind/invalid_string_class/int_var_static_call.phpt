===description===
Int var static call
===file===
<?php
$a = 5;
$a::bar();
===expect===
InvalidStringClass@3:1-3:3: Dynamic class instantiation requires string or class-string type, got '5'
