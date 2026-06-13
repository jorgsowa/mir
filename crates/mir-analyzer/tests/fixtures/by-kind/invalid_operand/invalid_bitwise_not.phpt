===description===
Invalid bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~new stdClass;
===expect===
InvalidOperand@2:7-2:19: Operator '~' not supported between 'stdClass' and ''
