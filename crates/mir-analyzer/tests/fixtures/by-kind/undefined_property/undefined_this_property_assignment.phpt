===description===
Undefined this property assignment
===file===
<?php
class A {
    public function fooFoo(): void {
        $this->foo = "cool";
//      ^^^^^^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property A::$foo is deprecated
    }
}
