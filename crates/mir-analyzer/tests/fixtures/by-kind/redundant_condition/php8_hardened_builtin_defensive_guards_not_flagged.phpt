===description===
Builtins whose failure return was removed in PHP 8 (`substr`, `mysqli_init`, `fgetcsv`) keep their narrowed type, but a defensive guard against the old failure value is not flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function substr_identical(string $s): string {
    $part = substr($s, 1);
    if ($part === false) {
        throw new \RuntimeException('substr failed');
    }
    /** @mir-check $part is string */
    return $part;
}

function substr_not_identical(string $s): string {
    $part = substr($s, 1);
    if ($part !== false) {
        /** @mir-check $part is string */
        return $part;
    }
    return '';
}

function substr_inline(string $s): string {
    if (substr($s, 1) === false) {
        return '';
    }
    return $s;
}

function substr_unchecked(string $s): string {
    $part = substr($s, 1);
    /** @mir-check $part is string */
    return $part;
}

function mysqli_guard(): \mysqli {
    $conn = mysqli_init();
    if ($conn === false) {
        throw new \RuntimeException('init failed');
    }
    /** @mir-check $conn is mysqli */
    return $conn;
}

/** @param resource $handle */
function csv_null_guard($handle): int {
    $row = fgetcsv($handle);
    if ($row === null) {
        return -1;
    }
    /** @mir-check $row is array|false */
    return $row === false ? 0 : count($row);
}
