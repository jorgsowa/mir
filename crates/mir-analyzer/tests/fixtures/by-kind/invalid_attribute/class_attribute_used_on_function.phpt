===description===
Class attribute used on function
===file===
<?php
namespace Foo;

#[Attribute(Attribute::TARGET_CLASS)]
class Table {
    public function __construct(public string $name) {}
}

#[Table("videos")]
function foo() : void {}
===expect===
InvalidAttribute@9:2-9:17: Attribute Table cannot be used on this target
