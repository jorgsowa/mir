===description===
Non nullable subclass param
===file===
<?php
class A {
    public function foo(?string $s): string {
        return $s !== null ? $s : "hello";
    }
}

class B extends A {
    public function foo(string $s): string {
        return $s;
    }
}
===expect===
Argument 1 of B::foo has wrong type \
===ignore===
TODO
