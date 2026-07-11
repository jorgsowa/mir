===description===
`!empty($arr['a'])` on a union of closed shapes must exclude the arms that
lack the key entirely, not just narrow the arm that has it — otherwise a
later access still sees the no-key arm and gets flagged as non-existent.
Mirrors the existing `isset($arr['a'])` closed-shape-arm exclusion.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{type: string, a: int}|array{type: string, b: string} $arr
 */
function test(array $arr): void {
    if (!empty($arr['a'])) {
        $val = $arr['a'];
        /** @mir-check $val is int */
        echo 1;
    }
}
===expect===
