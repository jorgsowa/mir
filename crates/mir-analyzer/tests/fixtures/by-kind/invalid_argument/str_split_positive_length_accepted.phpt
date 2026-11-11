===description===
A positive or omitted length is accepted by `str_split()` and `mb_str_split()`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param positive-int $size */
function chunks(string $value, int $size): void {
    str_split($value);
    str_split($value, 2);
    str_split($value, $size);
    mb_str_split($value);
    mb_str_split($value, 3);
    mb_str_split($value, $size);
    mb_str_split($value, max(1, strlen($value)));
}
