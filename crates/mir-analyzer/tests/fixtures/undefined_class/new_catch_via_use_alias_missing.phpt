===description===
new catch via use alias missing
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
UnusedVariable@4:5: Variable $x is never read
UndefinedClass@4:14: Class App\Model\MissingEntity does not exist
MissingThrowsDocblock@6:9: Exception Exception is thrown but not declared in @throws
UnusedVariable@7:13: Variable $e is never read
UndefinedClass@7:14: Class App\Model\MissingEntity does not exist
