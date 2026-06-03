===description===
Invalid boolean bitwise not
===file===
<?php
$a = ~true;
===expect===
InvalidOperand@2:7-2:11: Operator '~' not supported between 'true' and ''
