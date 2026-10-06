===description===
UndefinedDocblockClass fires when a `@psalm-import-type ... from` docblock
tag names a source class that does not exist.
===file===
<?php
/** @psalm-import-type UserId from NonExistentRepository */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentRepository' does not exist
class A {}
===expect===
