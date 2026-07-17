===description===
strpos()/array_search() compared with loose != false / == false narrow like
the strict !== false / === false handling; == false stays ambiguous (0 == false).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_strpos_loose_not_false(string $s): void {
    if (strpos($s, 'x') != false) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_false_loose_not_strpos(string $s): void {
    if (false != strpos($s, 'x')) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_strpos_loose_equal_false_not_narrowed(string $s): void {
    if (strpos($s, 'x') == false) {
        // Ambiguous: could be genuinely not found, or found at offset 0
        // (0 == false loosely) — must not narrow.
        /** @mir-check $s is string */
        $_ = $s;
    }
}

function test_array_search_loose_not_false_narrows_needle(string $mode): void {
    if (array_search($mode, ['read', 'write', 'append']) != false) {
        /** @mir-check $mode is "read"|"write"|"append" */
        $_ = $mode;
    }
}
===expect===
