===description===
class_exists guard suppresses UndefinedClass on instanceof in true branch
===file===
<?php
function test(mixed $x): bool {
    if (class_exists(\Optional\Pkg::class)) {
        return $x instanceof \Optional\Pkg;
    }
    return false;
}
===expect===
