===description===
Check variable in unknown class constructor
===file===
<?php
/** @suppress UndefinedClass */
new Missing($class_arg);
===expect===
UndefinedVariable@3:12-3:22: Variable $class_arg is not defined
