===description===
An anonymous class extending a nonexistent base must report UndefinedClass,
matching a named class's `extends` check.
===config===
suppress=UnusedVariable
===file===
<?php
$x = new class extends UndefinedBase {};
//                     ^^^^^^^^^^^^^ UndefinedClass: Class UndefinedBase does not exist
===expect===
