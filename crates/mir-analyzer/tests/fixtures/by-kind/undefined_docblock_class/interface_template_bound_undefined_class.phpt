===description===
UndefinedDocblockClass fires when an interface's own `@template T of Bound` bound names a
class that does not exist, matching a class's identical tag.
===file===
<?php
/** @template T of NonExistentBoundClass */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentBoundClass' does not exist
interface Box {}
===expect===
