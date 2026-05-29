===description===
Assert instance of multiple interfaces
===file===
<?php
class A {
    public function bar() : void {}
}
interface I1 {
    public function foo1(): void;
}
interface I2 {
    public function foo2(): void;
}
class B extends A implements I1, I2 {
    public function foo1(): void {}
    public function foo2(): void {}
}

function assertInstanceOfInterfaces(A $var): void {
    if (!$var instanceof I1 && !$var instanceof I2) {
        throw new Exception();
    }
}

function takesA(A $a): void {
    assertInstanceOfInterfaces($a);
    $a->bar();
    $a->foo1();
}
===expect===
MissingThrowsDocblock@18:9-18:31: Exception Exception is thrown but not declared in @throws
UndefinedMethod@25:5-25:15: Method A::foo1() does not exist
