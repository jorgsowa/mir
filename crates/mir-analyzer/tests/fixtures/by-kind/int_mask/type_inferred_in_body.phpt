===description===
Inside a function body, a parameter annotated @param int-mask<1, 2, 4> is
inferred as the full literal-int union 0|1|2|3|4|5|6|7.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param int-mask<1, 2, 4> $flags
 */
function check_flags(int $flags): void {
    /** @mir-check $flags is 0|1|2|3|4|5|6|7 */
    $_ = $flags;
}
===expect===
