===description===
UndefinedDocblockClass fires when a class-level `@method` magic docblock
tag's return type does not exist.
===file===
<?php
/** @method NonExistentReturnType getThing() */
class A {}
===expect===
UndefinedDocblockClass@2:0-2:47: Docblock type 'NonExistentReturnType' does not exist
