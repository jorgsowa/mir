===description===
Undefined var in bare callable
===config===
suppress=UnusedVariable
===file===
<?php
$fn = function(int $a): void{};
function a(callable $fn): void{
  $fn(++$a);
}
a($fn);
===expect===
UndefinedVariable@4:8-4:10: Variable $a is not defined
