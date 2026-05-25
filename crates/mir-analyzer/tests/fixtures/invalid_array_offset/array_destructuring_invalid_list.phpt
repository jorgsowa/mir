===description===
Array destructuring invalid list
===file===
<?php
$a = 42;

list($id1, $name1) = $a;
===expect===
InvalidArrayOffset
===ignore===
TODO
