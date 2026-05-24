===description===
checkVariableInUnknownClassConstructor
===file===
<?php
/** @psalm-suppress UndefinedClass */
new Missing($class_arg);
===expect===
PossiblyUndefinedVariable
===ignore===
TODO
