===description===
An anonymous class implementing a nonexistent interface must report
UndefinedClass, matching a named class's `implements` check.
===config===
suppress=UnusedVariable
===file===
<?php
$x = new class implements UndefinedIface {};
===expect===
UndefinedClass@2:26-2:40: Class UndefinedIface does not exist
