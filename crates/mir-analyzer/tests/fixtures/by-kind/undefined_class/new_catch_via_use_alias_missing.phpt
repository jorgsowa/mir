===description===
new catch via use alias missing
===config===
suppress=MissingThrowsDocblock,UnusedVariable,UnusedFunction
===file===
<?php
use App\Model\MissingEntity;
function wrap(): void {
    $x = new MissingEntity();
//           ^^^^^^^^^^^^^ UndefinedClass: Class App\Model\MissingEntity does not exist
    try {
        throw new \Exception();
    } catch (MissingEntity $e) {}
//           ^^^^^^^^^^^^^ UndefinedClass: Class App\Model\MissingEntity does not exist
}
===expect===
