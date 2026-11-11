===description===
`str_split()` throws for a length below 1, so its length is a `positive-int`.
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
    str_split($value, 0);
//                    ^ InvalidArgument: Argument $length of str_split() expects 'positive-int', got '0'
    str_split($value, -1);
//                    ^^ InvalidArgument: Argument $length of str_split() expects 'positive-int', got '-1'
}
