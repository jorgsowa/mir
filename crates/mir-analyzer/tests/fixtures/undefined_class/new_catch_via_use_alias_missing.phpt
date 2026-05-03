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
UnusedVariable@1:0: Variable $e is never read
UnusedVariable@4:4: Variable $x is never read
UndefinedClass@4:13: Class App\Model\MissingEntity does not exist
UndefinedClass@7:13: Class App\Model\MissingEntity does not exist
===ignore===
TODO
