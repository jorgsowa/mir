===description===
Explicit int cast on int variable - should not emit ImplicitFloatToIntCast

===config===
suppress=UnusedVariable
===file===
<?php
$x = 3;
$y = (int)$x;
//        ^^ RedundantCast: Casting '3' to 'int' is redundant

===expect===
