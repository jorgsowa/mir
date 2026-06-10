===description===
Check variable in unknown class constructor
===file===
<?php
/** @suppress UndefinedClass */
new Missing($class_arg);
===expect===
UndefinedVariable@3:13-3:23: Variable $class_arg is not defined
