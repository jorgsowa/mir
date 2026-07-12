===description===
An anonymous class extending a nonexistent base must report UndefinedClass,
matching a named class's `extends` check.
===config===
suppress=UnusedVariable
===file===
<?php
$x = new class extends UndefinedBase {};
===expect===
UndefinedClass@2:23-2:36: Class UndefinedBase does not exist
