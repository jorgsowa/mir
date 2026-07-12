===description===
UndefinedDocblockClass fires when a function's `@throws` docblock names a
class that does not exist.
===file===
<?php
/**
 * @throws NonExistentException
 */
function risky(): void {
}
===expect===
UndefinedDocblockClass@5:9-5:14: Docblock type 'NonExistentException' does not exist
