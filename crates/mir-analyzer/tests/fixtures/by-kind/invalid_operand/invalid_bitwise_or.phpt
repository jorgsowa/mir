===description===
Invalid bitwise or
===config===
suppress=UnusedVariable
===file===
<?php
$a = "x" | new stdClass;
//   ^^^^^^^^^^^^^^^^^^ InvalidOperand: Operator '|' not supported between '"x"' and 'stdClass'
===expect===
