===description===
UndefinedThrowsDocblock fires when a function's `@throws` docblock names a
class that does not exist. It is a warning (shown by default) — the
info-level UndefinedDocblockClass remains for the other docblock tags.
===file===
<?php
/**
 * @throws NonExistentException
 */
function risky(): void {
}
===expect===
UndefinedThrowsDocblock@5:9-5:14: @throws class 'NonExistentException' does not exist
