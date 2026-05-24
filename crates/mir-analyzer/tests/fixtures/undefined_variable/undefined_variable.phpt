===description===
undefinedVariable
===file===
<?php
$a = function() use ($i) {};
===expect===
UndefinedVariable@2:21: Variable $i is not defined
