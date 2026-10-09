===description===
A sibling declaring the method (no `__call` anywhere) downgrades the atom
lacking it to Info PossiblyUndefinedMethod rather than suppressing it.
===file===
<?php
class ServiceA {
    public function reveal(): void {}
}
class ServiceB {
    public function doOther(): void {}
}
function test(ServiceA|ServiceB $service): void {
    $service->reveal();
//  ^^^^^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method ServiceB::reveal() might not exist
}
