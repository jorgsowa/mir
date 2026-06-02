===description===
Addition with class in weak mode
===file===
<?php
$a = "hi" + (new stdClass);
===expect===
InvalidOperand@2:6-2:27: Operator '+' not supported between '"hi"' and 'stdClass'
