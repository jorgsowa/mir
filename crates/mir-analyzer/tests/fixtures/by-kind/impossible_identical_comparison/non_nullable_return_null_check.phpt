===description===
Null check on the result of a function returning non-nullable string is dead; comparing to '' is fine.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sanitize(string $s): string { return trim($s); }
function nullable(string $s): ?string { return $s === '' ? null : $s; }
function test(string $input): void {
    $ref = sanitize($input);
    $a = $ref === null ? 'direct' : 'referral';
//       ^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
    $b = $ref === '' ? 'direct' : 'referral';
    $maybe = nullable($input);
    $c = $maybe === null ? 'direct' : 'referral';
    echo $a, $b, $c;
}
===expect===
