===description===
Abstract class method
===file===
<?php
abstract class A {
    abstract public function foo() : void;
}

class B extends A { }
===expect===
UnimplementedAbstractMethod@6:0-6:21: Class B must implement abstract method foo()
