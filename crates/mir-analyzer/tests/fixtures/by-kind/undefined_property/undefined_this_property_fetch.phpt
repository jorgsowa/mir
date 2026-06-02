===description===
Undefined this property fetch
===file===
<?php
class A {
    public function fooFoo(): void {
        echo $this->foo;
    }
}
===expect===
UndefinedProperty@4:21-4:24: Property A::$foo does not exist
