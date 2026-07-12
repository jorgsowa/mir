===description===
UndefinedDocblockClass fires when a `@template T of Bound` bound names a
class that does not exist.
===file===
<?php
/** @template T of NonExistentBoundClass */
class Box {}
===expect===
UndefinedDocblockClass@2:0-2:43: Docblock type 'NonExistentBoundClass' does not exist
