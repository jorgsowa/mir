===description===
array_filter without a callback removes null/falsy from the value type
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string|null $s @param array<int, string|null> $arr */
function run(?string $s, array $arr): void {
    $a = array_filter([$s]);
    /** @mir-check $a is array<0, non-empty-string> */
    $b = array_filter($arr);
    /** @mir-check $b is array<int, non-empty-string> */
    $c = array_filter($arr, null);
    /** @mir-check $c is array<int, non-empty-string> */
    $d = array_filter($arr, null, ARRAY_FILTER_USE_KEY);
    /** @mir-check $d is array<int, non-empty-string> */
    $e = array_filter(['a' => $s, 'b' => 1]);
    /** @mir-check $e is array{a?: non-empty-string, b?: 1} */
    $f = array_filter($arr, fn($v) => $v !== '');
    /** @mir-check $f is array<int, string|null> */
}
===expect===
