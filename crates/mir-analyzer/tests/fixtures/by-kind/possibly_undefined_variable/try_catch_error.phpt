===description===
try catch error
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(): string {
    try {
        $result = strtolower("hi");
    } catch (\Exception $e) {
        // does not assign $result
    }
    return $result;
//         ^^^^^^^ PossiblyUndefinedVariable: Variable $result might not be defined
}
