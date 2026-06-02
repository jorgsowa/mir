===description===
Undefined variable static call
===file===
<?php
$foo::bar();
===expect===
UndefinedVariable@2:1-2:5: Variable $foo is not defined
