===description===
Negative control for the L27 fix: excluding the synthetic func_get_args()
`...` param from the comparison must not hide a GENUINE fewer-real-params
violation — the child here really does drop a required parameter.
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {
        func_get_args();
    }
}

class B extends A {
    public function fooFoo(int $a): void {
    }
}
===expect===
MethodSignatureMismatch@9:4-9:42: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent A::foofoo() (2)
