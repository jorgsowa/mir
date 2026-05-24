===description===
additionWithClassInWeakMode
===file===
<?php
$a = "hi" + (new stdClass);
===expect===
InvalidOperand
===ignore===
TODO
