===description===
interface_exists guard suppresses UndefinedClass in true branch
===file===
<?php
function test(): void {
    if (interface_exists(\Optional\Iface::class)) {
        $x = new class implements \Optional\Iface {};
    }
}
===expect===
