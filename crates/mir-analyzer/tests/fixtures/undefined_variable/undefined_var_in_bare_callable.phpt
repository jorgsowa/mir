===description===
undefinedVarInBareCallable
===file===
<?php
$fn = function(int $a): void{};
function a(callable $fn): void{
  $fn(++$a);
}
a($fn);
===expect===
UndefinedVariable@4:8: Variable $a is not defined
