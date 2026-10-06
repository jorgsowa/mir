===description===
Missing property type
===file===
<?php
class A {
    public $foo = null;
//  ^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property A::$foo has no type annotation

    public function assignToFoo(): void {
        $this->foo = 5;
    }
}
