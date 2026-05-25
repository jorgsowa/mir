===description===
Get attributes on class with non class attribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

class Foo {}

$r = new ReflectionClass(Foo::class);
$r->getAttributes(Attr::class);

===expect===
InvalidAttribute
===ignore===
TODO
