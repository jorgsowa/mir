===description===
`$row === []` / `$row !== []` on a union of a shape and `array{}` must drop
the empty-shape arm from the non-empty branch, so the key access is valid.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{algorithm: non-empty-string}|array{} $row */
function early_return(array $row): string {
    if ($row === []) {
        return '';
    }
    $v = $row;
    /** @mir-check $v is array{algorithm: non-empty-string} */
    return $row['algorithm'];
}

/** @param array{algorithm: non-empty-string}|array{} $row */
function not_identical(array $row): string {
    if ($row !== []) {
        return $row['algorithm'];
    }
    return '';
}

/** @param array{algorithm: non-empty-string}|array{} $row */
function else_branch(array $row): string {
    if ($row === []) {
        $v = $row;
        /** @mir-check $v is array{} */
        return '';
//      ^^^^^^^^^^ TypeCheckMismatch: Type of $v is expected to be array{}, got array{'algorithm': non-empty-string}|array{}
    } else {
        return $row['algorithm'];
    }
}

/** @param array{algorithm: non-empty-string}|array{} $row */
function reversed_operands(array $row): string {
    if ([] === $row) {
        return '';
    }
    return $row['algorithm'];
}

/** @param array{algorithm: non-empty-string}|array{} $row */
function count_check(array $row): string {
    if (count($row) === 0) {
        return '';
    }
    return $row['algorithm'];
}
