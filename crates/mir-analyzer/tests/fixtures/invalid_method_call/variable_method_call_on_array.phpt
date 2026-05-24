===description===
variableMethodCallOnArray
===file===
<?php
$arr = [];
$b = "foo";
$arr->$b();
===expect===
InvalidMethodCall
===ignore===
TODO
