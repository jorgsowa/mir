===description===
!isset short-circuit with || operator — assignment expression as RHS
Variable in assignment RHS of || should be narrowed from !isset() in LHS
===config===
suppress=UnusedVariable
===file===
<?php
$x = !isset($y) || ($y = null);
// After fix: $y in assignment RHS should be narrowed as defined
===expect===
