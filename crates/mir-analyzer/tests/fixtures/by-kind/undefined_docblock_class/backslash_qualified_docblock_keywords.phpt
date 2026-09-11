===description===
Docblock keywords written with a leading backslash (`\int`,
`\boolean`, `\interface-string`) are still keywords, not
fully-qualified class names: the keyword lookup strips one leading
backslash before the table check, and the same applies inside generic
type arguments (`\int-mask<1, 2, 4>`). Each such spelling is also
reported as an InvalidDocblockType warning.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \int $a
 * @param \boolean $b
 * @return \interface-string
 */
function f($a, $b): string {
    return "x";
}

/** @return \int-mask<1, 2, 4> */
function flags() {
    return 3;
}

===expect===
InvalidDocblockType@3:10-3:14: Invalid docblock type: @param backslash-qualified non-class type '\int' is not a fully qualified name
InvalidDocblockType@4:10-4:18: Invalid docblock type: @param backslash-qualified non-class type '\boolean' is not a fully qualified name
InvalidDocblockType@5:11-5:28: Invalid docblock type: @return backslash-qualified non-class type '\interface-string' is not a fully qualified name
InvalidDocblockType@11:12-11:30: Invalid docblock type: @return backslash-qualified non-class type '\int-mask<1, 2, 4>' is not a fully qualified name
