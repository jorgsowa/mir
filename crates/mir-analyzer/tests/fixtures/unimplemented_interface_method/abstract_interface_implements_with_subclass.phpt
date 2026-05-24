===description===
abstractInterfaceImplementsWithSubclass
===file===
<?php
interface I {
    public function fnc() : void;
}

abstract class A implements I {}

class B extends A {}
===expect===
UnimplementedInterfaceMethod
===ignore===
TODO
