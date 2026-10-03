===description===
Regression (laravel/framework): an assignment in a `match` arm condition defines a
variable usable in the arm body (ComponentTagCompiler). mir now analyzes arm
conditions in the arm context, so the assignment is registered and no longer
emits UndefinedVariable.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function guess(string $name): string {
    return match (true) {
        ($guess = strtolower($name)) !== '' => $guess,
        default => 'fallback',
    };
}
===expect===
