===description===
Docblock keywords written with a leading backslash (`\int`,
`\boolean`, `\interface-string`) are still keywords, not
fully-qualified class names: the keyword lookup strips one leading
backslash before the table check, and the same applies inside generic
type arguments (`\int-mask<1, 2, 4>`).
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
