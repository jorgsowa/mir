===description===
gettype switch arm unreachable for the argument's inferred type
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function scope(int $n): void {
    switch (gettype($n)) {
        case "integer":
            break;
        case "string":
//           ^^^^^^^^ UnevaluatedCode: Unevaluated code: gettype() of int never returns "string"
            break;
    }
}
===expect===
