===description===
iconv's on-error `false` is stripped from the inferred type for string input, but a defensive guard, cast or negation against it must not report ImpossibleIdenticalComparison, RedundantCast or RedundantCondition, and everyday unchecked use keeps the narrowed `string`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function identical_guard(string $s): string {
    $result = iconv('UTF-8', 'ASCII//TRANSLIT', $s);
    if ($result === false) {
        throw new \RuntimeException('conversion failed');
    }
    /** @mir-check $result is string */
    return $result;
}

function not_identical_guard(string $s): string {
    $result = iconv('UTF-8', 'ASCII//TRANSLIT', $s);
    if ($result !== false) {
        /** @mir-check $result is string */
        return $result;
    }
    return '';
}

function inline_guard(string $s): string {
    if (iconv('UTF-8', 'ASCII', $s) === false) {
        return '';
    }
    return $s;
}

function cast_guard(string $s): string {
    return (string) iconv('UTF-8', 'ASCII', $s);
}

function ternary_guard(string $s): string {
    $result = iconv('UTF-8', 'ASCII', $s);
    return $result === false ? '' : $result;
}

function unchecked(string $s): string {
    $result = iconv('UTF-8', 'ASCII', $s);
    /** @mir-check $result is string */
    return $result;
}

function non_empty_subject(): string {
    $result = iconv('UTF-8', 'ASCII', 'abc');
    if ($result === false) {
        return '';
    }
    return $result;
}
