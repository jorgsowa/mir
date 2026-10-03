===description===
Negative control: a `for` condition that no iteration can ever change is still flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(\Throwable $e): void {
    for ($i = 0, $c = $e; $c !== null; $i++) {
//                        ^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'Throwable' and 'null' is always true — these types can never be identical
        if ($i > 3) {
            break;
        }
    }
}
===expect===
