===description===
Negative control for the test-double union fix: a union with no `__call`
anywhere must still be flagged — the sibling-`__call` leniency must not
become a blanket union relaxation.
===file===
<?php
class ServiceA {
    public function doWork(): void {}
}
class ServiceB {
    public function doOther(): void {}
}
function test(ServiceA|ServiceB $service): void {
    $service->reveal();
}
===expect===
UndefinedMethod@9:4-9:22: Method ServiceA::reveal() does not exist
