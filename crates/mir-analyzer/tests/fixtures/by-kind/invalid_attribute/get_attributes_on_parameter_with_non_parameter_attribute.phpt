===description===
Get attributes on parameter with non parameter attribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

function foo(int $bar): void {}

$r = new ReflectionParameter("foo", "bar");
$r->getAttributes(Attr::class);

===expect===
