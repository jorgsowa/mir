===description===
Check variable in unknown class constructor
===file===
<?php
/** @suppress UndefinedClass */
new Missing($class_arg);
===expect===
PossiblyUndefinedVariable
===ignore===
TODO
