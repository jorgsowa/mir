===description===
`??` binds looser than `===`, so `$a ?? null === 'x'` compares null to 'x'; parenthesised form is fine.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $map, string $key): void {
    $bad = $map[$key] ?? null === 'foo';
//                       ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'null' and '"foo"' is always false — these types can never be identical
    $ok = ($map[$key] ?? null) === 'foo';
    echo $bad, $ok;
}
