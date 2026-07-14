===description===
Invalid bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~new stdClass;
===expect===
InvalidOperand@2:6-2:18: Operator '~' not supported for operand of type 'stdClass'
