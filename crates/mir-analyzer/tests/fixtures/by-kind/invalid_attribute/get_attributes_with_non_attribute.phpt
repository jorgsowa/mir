===description===
Get attributes with non attribute
===config===
suppress=UnusedParam
===file===
<?php
class NonAttr {}

function foo(int $bar): void {}

$r = new ReflectionParameter("foo", "bar");
$r->getAttributes(NonAttr::class);

===expect===
