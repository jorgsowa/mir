===description===
`@throws` annotations report undefined classes.
===file===
<?php
/**
 * @throws NonExistentException
 */
function risky(): void {
//       ^^^^^ UndefinedThrowsDocblock: @throws class 'NonExistentException' does not exist
}
===expect===
