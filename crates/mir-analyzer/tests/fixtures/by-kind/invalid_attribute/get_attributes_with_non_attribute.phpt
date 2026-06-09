===description===
Get attributes with non attribute
===file===
<?php
class NonAttr {}

function foo(int $bar): void {}

$r = new ReflectionParameter("foo", "bar");
$r->getAttributes(NonAttr::class);

===expect===
