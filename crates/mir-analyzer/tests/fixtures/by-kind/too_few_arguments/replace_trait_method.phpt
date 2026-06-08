===description===
Replace trait method
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
MethodSignatureMismatch@13:4-13:47: Method C::foo() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
