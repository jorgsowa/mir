===description===
Invalid bitwise or
===file===
<?php
$a = "x" | new stdClass;
===expect===
InvalidOperand
===ignore===
TODO
