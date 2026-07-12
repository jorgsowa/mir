===description===
UndefinedDocblockClass fires when a class-level `@property` magic docblock
tag names a type that does not exist.
===file===
<?php
/** @property NonExistentPropertyType $foo */
class A {}
===expect===
UndefinedDocblockClass@2:0-2:45: Docblock type 'NonExistentPropertyType' does not exist
