===description===
Undefined var in bare callable
===config===
suppress=UnusedVariable
===file===
<?php
$fn = function(int $a): void{};
function a(callable $fn): void{
  $fn(++$a);
//      ^^ UndefinedVariable: Variable $a is not defined
}
a($fn);
===expect===
