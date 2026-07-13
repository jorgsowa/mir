===description===
A private method called only from a trait's own method body via self::
(where the method is supplied by the composing class) must not be
reported unused — parity with the $this->helper() instance-call case.
===config===
suppress=
===file===
<?php
trait T {
    public function pub(): void {
        self::helper();
    }
}

class Foo {
    use T;

    private static function helper(): void {}
}

(new Foo())->pub();
===expect===
