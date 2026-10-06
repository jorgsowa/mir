===description===
UndefinedDocblockClass fires when a class-level `@method` magic docblock
tag's parameter type does not exist.
===file===
<?php
/** @method void setThing(NonExistentParamType $thing) */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentParamType' does not exist
class A {}
