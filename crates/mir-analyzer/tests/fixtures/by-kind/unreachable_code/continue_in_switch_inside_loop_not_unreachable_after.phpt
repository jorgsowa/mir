===description===
A bare `continue;` in a switch case, nested inside a loop, exits only the
switch (PHP semantics) — code after the switch, still inside the loop body,
must not be flagged unreachable.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(array $items): void {
    foreach ($items as $item) {
        switch ($item) {
            case 1:
                continue;
            default:
                return;
        }
        echo "after switch";
    }
}
===expect===
