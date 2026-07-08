===description===
interface method fulfilled only via a `use Trait { orig as alias; }` rename is not reported
===config===
suppress=UnusedParam
===file===
<?php
trait A {
    public function hello(int $x): string { return "A"; }
}
trait B {
    public function hello(string $x): string { return "B"; }
}
class C {
    use A, B {
        A::hello insteadof B;
        B::hello as helloB;
    }
}
interface Greets {
    public function hello(int $x): string;
    public function helloB(string $x): string;
}
class D extends C implements Greets {}
===expect===
