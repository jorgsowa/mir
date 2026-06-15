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
UndefinedDocblockClass@6:9-6:12: Docblock type 'array<' does not exist
