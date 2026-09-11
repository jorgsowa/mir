===description===
`@throws A|B` union tag: only the nonexistent member fires UndefinedThrowsDocblock, not a garbled combined name.
===file===
<?php
final class KnownException extends \RuntimeException {}

/**
 * @throws KnownException|MissingException
 */
function risky(): void {
}
===expect===
UndefinedThrowsDocblock@7:9-7:14: @throws class 'MissingException' does not exist
