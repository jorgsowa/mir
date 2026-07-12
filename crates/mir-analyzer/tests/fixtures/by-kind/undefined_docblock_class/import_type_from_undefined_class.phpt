===description===
UndefinedDocblockClass fires when a `@psalm-import-type ... from` docblock
tag names a source class that does not exist.
===file===
<?php
/** @psalm-import-type UserId from NonExistentRepository */
class A {}
===expect===
UndefinedDocblockClass@2:0-2:59: Docblock type 'NonExistentRepository' does not exist
