===description===
A backslash-qualified keyword in `@throws` gets the backslash warning but
must not also be treated as an undefined class (docblock keywords are
filtered before the class-existence check); a backslash-qualified real
class is a valid fully qualified name and stays quiet.
===file===
<?php
/**
 * @throws \int
 * @throws \RuntimeException
 */
function risky(): void {
}
===expect===
InvalidDocblockType@3:11-3:15: Invalid docblock type: @throws backslash-qualified non-class type '\int' is not a fully qualified name
