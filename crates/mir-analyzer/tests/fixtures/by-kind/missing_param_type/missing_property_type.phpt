===description===
Missing property type
===file===
<?php
class A {
    public $foo = null;

    public function assignToFoo(): void {
        $this->foo = 5;
    }
}
===expect===
MissingPropertyType@3:5-3:23: Property A::$foo has no type annotation
