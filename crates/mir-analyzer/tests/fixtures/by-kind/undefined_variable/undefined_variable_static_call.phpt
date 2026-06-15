===description===
Undefined variable static call
===file===
<?php
$foo::bar();
===expect===
UndefinedVariable@2:0-2:4: Variable $foo is not defined
