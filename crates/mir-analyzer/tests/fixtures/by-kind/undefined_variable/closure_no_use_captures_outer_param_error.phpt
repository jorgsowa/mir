===description===
closure no use captures outer param error
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outer(string $x): callable {
    return function(): string {
        return $x;
//             ^^ UndefinedVariable: Variable $x is not defined
    };
}
===expect===
