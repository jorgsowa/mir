===description===
tooFewArgumentsToAttributeConstructor
===file===
<?php
namespace Foo;

#[Attribute(Attribute::TARGET_CLASS)]
class Table {
    public function __construct(public string $name) {}
}

#[Table()]
class Video {}
===expect===
TooFewArguments
===ignore===
TODO
