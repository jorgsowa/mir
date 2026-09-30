===description===
Undefined this property fetch
===file===
<?php
class A {
    public function fooFoo(): void {
        echo $this->foo;
//                  ^^^ UndefinedProperty: Property A::$foo does not exist
    }
}
===expect===
