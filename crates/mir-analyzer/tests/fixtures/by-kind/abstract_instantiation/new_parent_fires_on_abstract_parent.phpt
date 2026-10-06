===description===
new parent() inside a child class fires AbstractInstantiation when the parent is abstract.
===file===
<?php
abstract class AbstractBase {}
class Child extends AbstractBase {
    public function test(): void {
        new parent();
//          ^^^^^^ AbstractInstantiation: Cannot instantiate abstract class AbstractBase
    }
}
