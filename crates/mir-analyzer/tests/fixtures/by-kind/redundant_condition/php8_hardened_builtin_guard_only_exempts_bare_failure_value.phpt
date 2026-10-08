===description===
The exemption covers only a bare `false`/`null` comparison: any other literal compared against the result is still impossible.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function other_literal(string $s): bool {
    $part = substr($s, 1);
    return $part === 1;
//         ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and '1' is always false — these types can never be identical
}
