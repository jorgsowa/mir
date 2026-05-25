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
MissingPropertyType
===ignore===
TODO
