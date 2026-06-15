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
UndefinedClass@4:13-4:26: Class App\Model\MissingEntity does not exist
UndefinedClass@7:13-7:26: Class App\Model\MissingEntity does not exist
