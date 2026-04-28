===file===
<?php
use App\Model\MissingEntity;
function wrap(): void {
    $x = new MissingEntity();
    try {
        throw new \Exception();
    } catch (MissingEntity $e) {}
}
===expect===
UnusedVariable: Variable $x is never read
UnusedVariable: Variable $e is never read
UndefinedClass: Class App\Model\MissingEntity does not exist
UndefinedClass: Class App\Model\MissingEntity does not exist
