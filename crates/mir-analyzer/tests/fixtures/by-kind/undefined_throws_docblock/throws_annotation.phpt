===description===
`@throws` annotations report undefined classes.
===file===
<?php
/**
 * @throws NonExistentException
 */
function risky(): void {
}
===expect===
UndefinedThrowsDocblock@5:9-5:14: @throws class 'NonExistentException' does not exist
