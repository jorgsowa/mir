===description===
`$x == 'lit'` with a non-empty, non-numeric literal narrows a union of
strings, ints, floats and null like `===`: no int, float or null loosely
equals it. A union with `bool` (`true == 'lit'`) or a numeric literal
(`5 == '5'`) isn't narrowed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'foo'|'bar'|int $key */
function keys(string|int $key): void {
    if ($key == 'foo') {
        /** @mir-check $key is "foo" */
        $_ = $key;
    }
    if ('foo' == $key || $key == 'bar') {
        return;
    }
    /** @mir-check $key is int */
    $_ = $key;
}

function general(?string $s): void {
    if ($s == 'yes') {
        /** @mir-check $s is "yes" */
        $_ = $s;
    }
}

function withBool(string|bool $v): void {
    if ($v == 'yes') {
        /** @mir-check $v is string|bool */
        $_ = $v;
    }
}

function numericLiteral(string|int $v): void {
    if ($v == '5') {
        /** @mir-check $v is string|int */
        $_ = $v;
    }
}
