===description===
AbstractInstantiation fires when an abstract class is instantiated inside a class method body.
===file===
<?php
abstract class Foo {}
class Bar {
    public function test(): void {
        new Foo();
    }
}
===expect===
AbstractInstantiation@5:12-5:15: Cannot instantiate abstract class Foo
