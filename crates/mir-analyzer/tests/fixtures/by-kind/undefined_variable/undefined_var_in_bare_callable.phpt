===description===
Undefined var in bare callable
===file===
<?php
$fn = function(int $a): void{};
function a(callable $fn): void{
  $fn(++$a);
}
a($fn);
===expect===
UndefinedVariable@4:9-4:11: Variable $a is not defined
