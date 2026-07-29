===description===
Negative control: the sibling atom having the method directly (with no
`__call` anywhere in the union) must NOT suppress UndefinedMethod on the atom
that lacks it — the fix is scoped to a magic-`__call` sibling specifically,
not a blanket "some union member has it" relaxation.
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
}
===expect===
UndefinedMethod@9:4-9:22: Method ServiceB::reveal() does not exist
