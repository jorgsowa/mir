===description===
Unimplemented interface method
===file===
<?php
interface A {
    public function fooFoo() : void;
}

class B implements A { }
//<^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class B must implement A::fooFoo() from interface
===expect===
