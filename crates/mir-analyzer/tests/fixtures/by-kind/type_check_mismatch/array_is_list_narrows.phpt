===description===
array_is_list() narrows the argument to a list type in the true branch.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php

/** @param array<int, string> $arr */
function test_int_keyed(array $arr): void {
    if (array_is_list($arr)) {
        /** @mir-check $arr is list<string> */
        $_ = $arr;
    }
}

/** @param non-empty-array<int, string> $arr */
function test_non_empty_int_keyed(array $arr): void {
    if (array_is_list($arr)) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}
