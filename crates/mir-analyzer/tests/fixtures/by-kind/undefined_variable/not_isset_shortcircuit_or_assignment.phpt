===description===
!isset short-circuit with || operator — assignment in expression
Variable in function call on RHS of assignment should be narrowed from !isset() in condition
===config===
suppress=MissingParamType,UnusedVariable
===file===
<?php
function doSomething($x): void { echo $x; }
$result = !isset($x) || doSomething($x);
// After fix: $x should be narrowed as defined in RHS
===expect===
