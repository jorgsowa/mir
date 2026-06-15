===description===
Invalid bitwise or
===config===
suppress=UnusedVariable
===file===
<?php
$a = "x" | new stdClass;
===expect===
InvalidOperand@2:5-2:23: Operator '|' not supported between '"x"' and 'stdClass'
