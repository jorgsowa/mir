===description===
suppressed via psalm suppress
===file===
<?php
/**
 * @psalm-suppress InvalidDocblock
 * @return array<
 */
function foo(): mixed { return []; }
//       ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
