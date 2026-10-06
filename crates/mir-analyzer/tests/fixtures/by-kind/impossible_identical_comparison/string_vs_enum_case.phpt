===description===
A string property can never be === to an enum case; converting with from() is fine.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Kind: string { case Admin = 'admin'; }
final class Row { public string $type = 'admin'; }
function test(Row $row): void {
    if ($row->type === Kind::Admin) {}
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'Kind' is always false — these types can never be identical
    if (Kind::from($row->type) === Kind::Admin) {}
}
function negated(string $raw): void {
    if ($raw !== Kind::Admin) {}
//      ^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'string' and 'Kind' is always true — these types can never be identical
}
function safe(string $raw): void {
    if (Kind::tryFrom($raw) === Kind::Admin) {}
}
