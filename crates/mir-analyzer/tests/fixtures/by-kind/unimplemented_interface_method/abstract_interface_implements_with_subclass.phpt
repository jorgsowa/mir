===description===
Abstract interface implements with subclass
===file===
<?php
interface I {
    public function fnc() : void;
}

abstract class A implements I {}

class B extends A {}
===expect===
UnimplementedInterfaceMethod@8:0-8:20: Class B must implement I::fnc() from interface
