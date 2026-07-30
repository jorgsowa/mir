===description===
Negative control for the union-receiver arity fix: a union with no `__call`
anywhere must still flag a genuine arity mismatch on the atom that declares
the method — the sibling-`__call` leniency must not become a blanket union
relaxation.
===file===
<?php
class ServiceA {
    public function doSomething(int $a, int $b): void {
        echo $a + $b;
    }
}
class ServiceB {
    public function doSomething(int $a, int $b): void {
        echo $a + $b;
    }
}
function test(ServiceA|ServiceB $service): void {
    $service->doSomething(1);
}
===expect===
TooFewArguments@13:4-13:28: Too few arguments for doSomething(): expected 2, got 1
