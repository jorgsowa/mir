===description===
Widening cast from int to float - should not be redundant or error

===config===
suppress=UnusedVariable
===file===
<?php
$x = 3;
$y = (float)$x;

===expect===
