===description===
Regression (laravel/framework): an assignment in a `match` arm condition defines a
variable usable in the arm body (ComponentTagCompiler). mir now analyzes arm
conditions in the arm context, so the assignment is registered and no longer
emits UndefinedVariable.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
function guess(string $name): string {
    return match (true) {
        ($guess = strtolower($name)) !== '' => $guess,
        default => 'fallback',
    };
}
===expect===
