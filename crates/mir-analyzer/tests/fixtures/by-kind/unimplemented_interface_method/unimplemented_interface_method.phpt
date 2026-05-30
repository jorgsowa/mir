===description===
Unimplemented interface method
===file===
<?php
interface A {
    public function fooFoo() : void;
}

class B implements A { }
===expect===
UnimplementedInterfaceMethod@6:0-6:24: Class B must implement A::fooFoo() from interface
