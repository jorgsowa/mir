===description===
Check variable in unknown class constructor
===file===
<?php
/** @suppress UndefinedClass */
new Missing($class_arg);
//          ^^^^^^^^^^ UndefinedVariable: Variable $class_arg is not defined
