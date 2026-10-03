===description===
An anonymous class implementing a nonexistent interface must report
UndefinedClass, matching a named class's `implements` check.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = new class implements UndefinedIface {};
//                        ^^^^^^^^^^^^^^ UndefinedClass: Class UndefinedIface does not exist
===expect===
