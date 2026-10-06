===description===
Abstract interface implements with subclass
===file===
<?php
interface I {
    public function fnc() : void;
}

abstract class A implements I {}

class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class B must implement I::fnc() from interface
