===description===
Abstract class method
===file===
<?php
abstract class A {
    abstract public function foo() : void;
}

class B extends A { }
===expect===
UnimplementedAbstractMethod
===ignore===
TODO
