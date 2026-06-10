===description===
Undefined this property assignment
===ignore===
TODO
===file===
<?php
class A {
    public function fooFoo(): void {
        $this->foo = "cool";
    }
}
===expect===
