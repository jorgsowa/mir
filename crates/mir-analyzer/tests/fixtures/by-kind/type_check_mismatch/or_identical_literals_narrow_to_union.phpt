===description===
A disjunction of identity comparisons against literals narrows the subject to the union of those literals.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function strings(string $x): void {
    if ($x === 'a' || $x === 'b') {
        /** @mir-check $x is 'a'|'b' */
        $_ = $x;
    } else {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

function ints(int $x): void {
    if ($x === 1 || 2 === $x || $x === 3) {
        /** @mir-check $x is 1|2|3 */
        $_ = $x;
    }
}

function from_mixed(mixed $x): void {
    if ($x === 'a' || $x === 1) {
        /** @mir-check $x is 'a'|1 */
        $_ = $x;
    }
}

function guard_clause(string $x): void {
    if (!($x === 'a' || $x === 'b')) {
        return;
    }
    /** @mir-check $x is 'a'|'b' */
    $_ = $x;
}

function negated_guard_clause(string $x): void {
    if ($x !== 'a' && $x !== 'b') {
        return;
    }
    /** @mir-check $x is 'a'|'b' */
    $_ = $x;
}

function different_subjects(string $x, string $y): void {
    if ($x === 'a' || $y === 'b') {
        /** @mir-check $x is string */
        $_ = $x;
        /** @mir-check $y is string */
        $_ = $y;
    }
}

function with_null(?string $x): void {
    if ($x === 'a' || $x === null) {
        /** @mir-check $x is 'a'|null */
        $_ = $x;
    }
}

function non_comparison_disjunct(?string $x, bool $flag): void {
    if ($x === 'a' || $flag) {
        /** @mir-check $x is string|null */
        $_ = $x;
    }
}
