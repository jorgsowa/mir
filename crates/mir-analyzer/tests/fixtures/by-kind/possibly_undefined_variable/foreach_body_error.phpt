===description===
foreach body error
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(array $items): string {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
//         ^^^^^ PossiblyUndefinedVariable: Variable $last might not be defined
}
===expect===
