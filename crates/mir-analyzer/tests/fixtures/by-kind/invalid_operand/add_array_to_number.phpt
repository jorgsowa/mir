===description===
Add array to number
===config===
suppress=UnusedVariable
===file===
<?php
$a = [1] + 1;
===expect===
InvalidOperand@2:5-2:12: Operator '+' not supported between 'array{0: 1}' and '1'
