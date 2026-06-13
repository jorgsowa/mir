===description===
True increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = true;
$a++;
===expect===
InvalidOperand@3:1-3:3: Operator '++' not supported between 'true' and ''
