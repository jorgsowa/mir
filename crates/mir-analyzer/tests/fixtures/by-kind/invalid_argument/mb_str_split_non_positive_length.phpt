===description===
`mb_str_split()` throws for a length below 1, so its length is a `positive-int`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function chunks(string $value): void {
    mb_str_split($value, 0);
//                       ^ InvalidArgument: Argument $length of mb_str_split() expects 'positive-int', got '0'
    mb_str_split($value, -3, 'UTF-8');
//                       ^^ InvalidArgument: Argument $length of mb_str_split() expects 'positive-int', got '-3'
}
