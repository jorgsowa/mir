===description===
class_exists guard suppresses UndefinedClass on static method call in true branch
===file===
<?php
function test(): void {
    if (class_exists(\Optional\Pkg::class)) {
        \Optional\Pkg::create();
    }
}
===expect===
