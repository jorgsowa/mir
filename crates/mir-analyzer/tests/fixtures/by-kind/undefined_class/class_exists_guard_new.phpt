===description===
class_exists guard with ::class suppresses UndefinedClass on new in true branch
===file===
<?php
function test(): void {
    if (class_exists(\Optional\Pkg::class)) {
        new \Optional\Pkg();
    }
}
===expect===
