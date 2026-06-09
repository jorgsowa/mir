===description===
Get attributes on property with non property attribute
===file===
<?php
#[Attribute(Attribute::TARGET_CLASS)]
class Attr {}

class Foo
{
    public string $bar = "baz";
}

$r = new ReflectionProperty(Foo::class, "bar");
$r->getAttributes(Attr::class);

===expect===
