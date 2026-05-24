===description===
possiblyFalseReference
===file===
<?php
class A {
    public function bar(): void {}
}

$a = rand(0, 1) ? new A : false;
$a->bar();
===expect===
PossiblyFalseReference
===ignore===
TODO
