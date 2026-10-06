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
//^^^^^^^^^^^^^^^ InvalidAttribute: Attribute Table cannot be used on this target
function foo() : void {}
