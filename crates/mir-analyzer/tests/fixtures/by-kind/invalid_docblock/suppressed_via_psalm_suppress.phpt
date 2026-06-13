===description===
suppressed via psalm suppress
===file===
<?php
/**
 * @psalm-suppress InvalidDocblock
 * @return array<
 */
function foo(): mixed { return []; }
===expect===
UndefinedDocblockClass@6:10-6:13: Docblock type 'array<' does not exist
