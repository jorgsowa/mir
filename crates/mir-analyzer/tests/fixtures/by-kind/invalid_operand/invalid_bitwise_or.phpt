===description===
Invalid bitwise or
===file===
<?php
$a = "x" | new stdClass;
===expect===
InvalidOperand@2:6-2:24: Operator '|' not supported between '"x"' and 'stdClass'
