===description===
=== true/false on a `bool` narrows to the specific literal, not the wide bool type.
`$x: bool; if ($x === false)` true-branch → `false`; `$x !== false` true-branch → `true`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param bool $x */
function test_eq_false(bool $x): void {
    if ($x === false) {
        /** @mir-check $x is false */
        $_ = $x;
    } else {
        /** @mir-check $x is true */
        $_ = $x;
    }
}

/** @param bool $x */
function test_neq_false(bool $x): void {
    if ($x !== false) {
        /** @mir-check $x is true */
        $_ = $x;
    } else {
        /** @mir-check $x is false */
        $_ = $x;
    }
}

/** @param bool $x */
function test_eq_true(bool $x): void {
    if ($x === true) {
        /** @mir-check $x is true */
        $_ = $x;
    } else {
        /** @mir-check $x is false */
        $_ = $x;
    }
}
