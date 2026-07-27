===description===
FP: a class's own method replacing a same-named CONCRETE trait method is a
composition-time precedence rule, not an override — PHP performs no
compatibility check between them (verified live: this file loads with no
fatal, `$this->foo()` inside the trait itself still calls the trait's own
0-arg copy, unaffected by the composing class's replacement).
===config===
suppress=UnusedParam
===file===
<?php
trait T {
    protected function foo() : void {}

    public function bat() : void {
        $this->foo();
    }
}

class C {
    use T;

    protected function foo(string $s) : void {}
}
===expect===
