===description===
`@param self::MODE_*` resolves to the union of the matching int constants, so comparing against one of them is not impossible.
===file===
<?php
class A {
    const MODE_KEEP = 1;
    const MODE_DROP = 2;
    const OTHER = 9;

    /** @param self::MODE_* $mode */
    public function f($mode): string {
        /** @mir-check $mode is 1|2 */
        if ($mode === self::MODE_DROP) {
            return 'd';
        }
        return 'k';
    }
}
