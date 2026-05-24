===description===
getAttributesOnMethodWithNonMethodAttribute
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
InvalidAttribute
===ignore===
TODO
