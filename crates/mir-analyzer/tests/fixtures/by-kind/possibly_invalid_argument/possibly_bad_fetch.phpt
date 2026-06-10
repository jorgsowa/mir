===description===
Possibly bad fetch
===file===
<?php
$a = rand(0, 5) > 3 ? "hello" : new stdClass;
echo $a->foo;
===expect===
UndefinedProperty@3:10-3:13: Property stdClass::$foo does not exist
