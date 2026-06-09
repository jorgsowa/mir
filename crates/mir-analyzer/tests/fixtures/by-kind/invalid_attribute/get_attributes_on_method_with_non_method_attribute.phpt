===description===
Get attributes on method with non method attribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

class Foo
{
    public function bar(): void {}
}

$r = new ReflectionMethod("Foo::bar");
$r->getAttributes(Attr::class);

===expect===
