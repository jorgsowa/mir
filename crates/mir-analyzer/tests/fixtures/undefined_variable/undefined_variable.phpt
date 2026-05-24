===description===
undefinedVariable
===file===
<?php
$a = function() use ($i) {};
===expect===
UndefinedVariable@2:22: Variable $i is not defined
