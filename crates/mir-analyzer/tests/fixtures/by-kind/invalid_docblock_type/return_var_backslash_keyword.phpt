===description===
`@return` and `@var` positions are checked the same way: backslash-qualified
type keywords are warnings.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @return \string
 * @var \bool $flag
 */
function f(): string {
    return "x";
}
===expect===
InvalidDocblockType@3:11-3:18: Invalid docblock type: @return backslash-qualified non-class type '\string' is not a fully qualified name
InvalidDocblockType@4:8-4:13: Invalid docblock type: @var backslash-qualified non-class type '\bool' is not a fully qualified name
