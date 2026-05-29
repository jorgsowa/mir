===description===
new catch via use alias missing
===config===
suppress=MissingThrowsDocblock,UnusedVariable,UnusedFunction
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
UndefinedClass@4:14-4:27: Class App\Model\MissingEntity does not exist
UndefinedClass@7:14-7:27: Class App\Model\MissingEntity does not exist
