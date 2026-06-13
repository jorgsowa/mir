===description===
Invalid bitwise or
===config===
suppress=UnusedVariable
===file===
<?php
$a = "x" | new stdClass;
===expect===
InvalidOperand@2:6-2:24: Operator '|' not supported between '"x"' and 'stdClass'
