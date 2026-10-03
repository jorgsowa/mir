===description===
Invalid catch class
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
try {
    $worked = true;
}
catch (A $e) {}
//     ^ InvalidCatch: Caught type 'A' does not extend Throwable
===expect===
