===description===
A numeric string is a valid unary `-`/`+` operand and must not be flagged.
===config===
suppress=UnusedVariable
===file===
<?php
$a = -"5";
$b = +"5";
===expect===
