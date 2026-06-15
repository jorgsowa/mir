===description===
Invalid boolean bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~true;
===expect===
InvalidOperand@2:6-2:10: Operator '~' not supported between 'true' and ''
