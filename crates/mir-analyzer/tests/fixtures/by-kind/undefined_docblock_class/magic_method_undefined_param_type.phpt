===description===
UndefinedDocblockClass fires when a class-level `@method` magic docblock
tag's parameter type does not exist.
===file===
<?php
/** @method void setThing(NonExistentParamType $thing) */
class A {}
===expect===
UndefinedDocblockClass@2:0-2:57: Docblock type 'NonExistentParamType' does not exist
