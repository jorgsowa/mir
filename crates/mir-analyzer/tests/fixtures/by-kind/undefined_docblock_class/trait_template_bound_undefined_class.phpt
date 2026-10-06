===description===
UndefinedDocblockClass fires when a trait's own `@template T of Bound` bound names a
class that does not exist, matching a class's identical tag.
===file===
<?php
/** @template T of NonExistentBoundClass */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentBoundClass' does not exist
trait Box {}
