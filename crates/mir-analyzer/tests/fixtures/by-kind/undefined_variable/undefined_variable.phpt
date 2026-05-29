===description===
Undefined variable
===file===
<?php
$a = function() use ($i) {};
===expect===
UndefinedVariable@2:22-2:24: Variable $i is not defined
