===description===
`@throws A|B` union tag: only the nonexistent member fires UndefinedThrowsDocblock, not a garbled combined name.
===file===
<?php
final class KnownException extends \RuntimeException {}

/**
 * @throws KnownException|MissingException
 */
function risky(): void {
//       ^^^^^ UndefinedThrowsDocblock: @throws class 'MissingException' does not exist
}
===expect===
