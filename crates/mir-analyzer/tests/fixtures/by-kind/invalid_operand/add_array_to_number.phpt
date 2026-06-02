===description===
Add array to number
===file===
<?php
$a = [1] + 1;
===expect===
InvalidOperand@2:6-2:13: Operator '+' not supported between 'array{0: 1}' and '1'
