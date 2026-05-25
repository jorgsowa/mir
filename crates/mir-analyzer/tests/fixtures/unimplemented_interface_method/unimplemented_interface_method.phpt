===description===
Unimplemented interface method
===file===
<?php
interface A {
    public function fooFoo() : void;
}

class B implements A { }
===expect===
UnimplementedInterfaceMethod
===ignore===
TODO
