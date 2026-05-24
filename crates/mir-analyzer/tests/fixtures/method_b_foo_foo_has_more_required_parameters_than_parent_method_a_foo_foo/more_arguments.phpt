===description===
moreArguments
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a, bool $b, array $c): void {

    }
}
===expect===
Method B::fooFoo has more required parameters than parent method A::fooFoo
===ignore===
TODO
