===description===
AbstractInstantiation fires when an abstract class is instantiated inside a class method body.
===file===
<?php
abstract class Foo {}
class Bar {
    public function test(): void {
        new Foo();
//          ^^^ AbstractInstantiation: Cannot instantiate abstract class Foo
    }
}
===expect===
