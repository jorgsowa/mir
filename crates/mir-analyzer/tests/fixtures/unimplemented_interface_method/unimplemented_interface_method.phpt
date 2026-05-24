===description===
unimplementedInterfaceMethod
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
