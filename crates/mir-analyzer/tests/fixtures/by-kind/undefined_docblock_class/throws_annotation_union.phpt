===description===
`@throws A|B` union tag: only the nonexistent member fires UndefinedDocblockClass, not a garbled combined name.
===file===
<?php
final class KnownException extends \RuntimeException {}

/**
 * @throws KnownException|MissingException
 */
function risky(): void {
}
===expect===
UndefinedDocblockClass@7:9-7:14: Docblock type 'MissingException' does not exist
