===description===
True increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = true;
$a++;
===expect===
InvalidOperand@3:0-3:2: Operator '++' not supported between 'true' and ''
