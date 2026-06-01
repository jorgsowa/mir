===description===
early return on !class_exists suppresses UndefinedClass after the guard
===file===
<?php
function test(): void {
    if (!class_exists(\Optional\Pkg::class)) {
        return;
    }
    new \Optional\Pkg();
}
===expect===
