===description===
UndefinedDocblockClass fires when a class-level `@method` magic docblock
tag's return type does not exist.
===file===
<?php
/** @method NonExistentReturnType getThing() */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentReturnType' does not exist
class A {}
