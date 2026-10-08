===description===
The iconv defensive-guard exemption covers only a bare `false`/`null` comparison: any other literal compared against the result is still impossible.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function other_literal(string $s): bool {
    $result = iconv('UTF-8', 'ASCII', $s);
    return $result === 1;
//         ^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and '1' is always false — these types can never be identical
}
