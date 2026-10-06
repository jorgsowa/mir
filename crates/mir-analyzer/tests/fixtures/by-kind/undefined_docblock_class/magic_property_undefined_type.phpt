===description===
UndefinedDocblockClass fires when a class-level `@property` magic docblock
tag names a type that does not exist.
===file===
<?php
/** @property NonExistentPropertyType $foo */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentPropertyType' does not exist
class A {}
