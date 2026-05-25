===description===
checkVariableInUnknownClassConstructor
===file===
<?php
/** @suppress UndefinedClass */
new Missing($class_arg);
===expect===
PossiblyUndefinedVariable
===ignore===
TODO
