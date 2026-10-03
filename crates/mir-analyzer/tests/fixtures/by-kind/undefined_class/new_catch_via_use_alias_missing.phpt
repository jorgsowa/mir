===description===
new catch via use alias missing
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
