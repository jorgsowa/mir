===description===
interface_exists guard suppresses UndefinedClass in true branch
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    if (interface_exists(\Optional\Iface::class)) {
        $x = new class implements \Optional\Iface {};
    }
}
===expect===
