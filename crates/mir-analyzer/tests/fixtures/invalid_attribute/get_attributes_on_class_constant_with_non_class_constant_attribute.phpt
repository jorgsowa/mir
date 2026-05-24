===description===
getAttributesOnClassConstantWithNonClassConstantAttribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

class Foo
{
    public const BAR = "baz";
}

$r = new ReflectionClassConstant(Foo::class, "BAR");
$r->getAttributes(Attr::class);

===expect===
InvalidAttribute
===ignore===
TODO
