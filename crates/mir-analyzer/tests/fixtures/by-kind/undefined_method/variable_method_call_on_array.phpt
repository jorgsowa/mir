===description===
Variable method call on array
===file===
<?php
$arr = [];
$b = "foo";
$arr->$b();
===expect===
InvalidMethodCall
===ignore===
TODO
