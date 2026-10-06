===description===
A variable known to be one of a closed literal union can never === a value
outside the union; both branches become known (true is always-true for !==,
always-false for ===).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <DocblockTypeContradiction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param 1|2|3 $n
 */
function test_ne_impossible($n): void {
    if ($n !== 5) {
//      ^^^^^^^^ ImpossibleIdenticalComparison: '!==' between '1|2|3' and '5' is always true — these types can never be identical
        $_ = $n; // always here
    }
}
