===description===
Require param
===config===
suppress=UnusedParam
===file===
<?php
interface I {
    function foo(bool $b = false): void;
}

class C implements I {
    public function foo(bool $b): void {}
}
===expect===
MethodSignatureMismatch@7:4-7:41: Method C::foo() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
