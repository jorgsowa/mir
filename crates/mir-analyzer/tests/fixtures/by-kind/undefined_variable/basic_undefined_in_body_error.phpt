===description===
Using a variable that was never assigned in the same scope reports UndefinedVariable.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(): string {
    return $result;
//         ^^^^^^^ UndefinedVariable: Variable $result is not defined
}
