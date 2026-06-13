===description===
Possibly bad fetch. stdClass permits arbitrary dynamic properties, so a `->foo`
fetch on a `string|stdClass` value is not reported as UndefinedProperty.
===file===
<?php
$a = rand(0, 5) > 3 ? "hello" : new stdClass;
echo $a->foo;
===expect===
