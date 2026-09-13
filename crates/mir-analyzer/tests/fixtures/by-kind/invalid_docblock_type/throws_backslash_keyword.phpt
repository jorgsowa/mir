===description===
Throws annotations reject backslash-qualified keywords but allow class names.
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
