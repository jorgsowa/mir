===description===
FP-C: grapheme_strlen returns int|false|null in stubs, but false/null only on
invalid UTF-8 input. Normal usage should not emit NullableReturnStatement.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function measure(string $s): int {
    return grapheme_strlen($s);
}

function measure_with_check(string $s): int {
    $len = grapheme_strlen($s);
    /** @mir-check $len is int */
    return $len;
}
===expect===
