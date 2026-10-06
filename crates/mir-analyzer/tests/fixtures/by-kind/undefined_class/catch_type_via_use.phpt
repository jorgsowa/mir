===description===
catch type via use
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
use Vendor\Missing\MyException;
function f(): void {
    try {
        throw new \Exception();
    } catch (MyException $e) {
//           ^^^^^^^^^^^ UndefinedClass: Class Vendor\Missing\MyException does not exist
    }
}
