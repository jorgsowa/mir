===description===
Without PREG_UNMATCHED_AS_NULL, an unmatched capture group is always the empty
string, never null — comparing it to null must still be flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function parseNumber(string $value): void {
    preg_match('/(?P<integral>\d+)(\.(?P<fraction>\d+))?/', $value, $matches);

    if ($matches['fraction'] === null) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
        echo "no fraction\n";
    }
}
===expect===
