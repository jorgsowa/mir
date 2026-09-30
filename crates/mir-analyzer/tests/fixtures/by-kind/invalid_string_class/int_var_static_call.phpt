===description===
Int var static call
===file===
<?php
$a = 5;
$a::bar();
//<^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got '5'
===expect===
