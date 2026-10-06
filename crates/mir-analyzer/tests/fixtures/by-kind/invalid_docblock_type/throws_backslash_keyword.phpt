===description===
Throws annotations reject backslash-qualified keywords but allow class names.
===file===
<?php
/**
 * @throws \int
//         ^^^^ InvalidDocblockType: Invalid docblock type: @throws backslash-qualified non-class type '\int' is not a fully qualified name
 * @throws \RuntimeException
 */
function risky(): void {
}
