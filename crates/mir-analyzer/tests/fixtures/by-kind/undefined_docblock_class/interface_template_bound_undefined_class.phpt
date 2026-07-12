===description===
UndefinedDocblockClass fires when an interface's own `@template T of Bound` bound names a
class that does not exist, matching a class's identical tag.
===file===
<?php
/** @template T of NonExistentBoundClass */
interface Box {}
===expect===
UndefinedDocblockClass@2:0-2:43: Docblock type 'NonExistentBoundClass' does not exist
