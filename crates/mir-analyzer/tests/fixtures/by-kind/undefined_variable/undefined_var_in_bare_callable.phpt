===description===
Undefined var in bare callable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = function(int $a): void{};
function a(callable $fn): void{
  $fn(++$a);
//      ^^ UndefinedVariable: Variable $a is not defined
}
a($fn);
===expect===
