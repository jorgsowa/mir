===description===
Method chain resolves intermediate type from same-file class definition
===file===
<?php
class A {
    public function getB(): B { return new B(); }
}
class B {
    public function getValue(): int { return 42; }
}
function test(A $a): void {
    $result = $a->getB()->getValue();
    /** @mir-check $result is int */
    $_ = $result;
}
===expect===
