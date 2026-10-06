===description===
UndefinedDocblockClass fires when a `@template T of Bound` bound names a
class that does not exist.
===file===
<?php
/** @template T of NonExistentBoundClass */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentBoundClass' does not exist
class Box {}
