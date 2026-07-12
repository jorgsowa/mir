===description===
UndefinedDocblockClass fires when a trait's own `@template T of Bound` bound names a
class that does not exist, matching a class's identical tag.
===file===
<?php
/** @template T of NonExistentBoundClass */
trait Box {}
===expect===
UndefinedDocblockClass@2:0-2:43: Docblock type 'NonExistentBoundClass' does not exist
