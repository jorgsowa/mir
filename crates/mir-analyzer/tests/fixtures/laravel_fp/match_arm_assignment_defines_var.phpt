===description===
Laravel FP (laravel/framework): an assignment in a `match` arm condition defines a
variable usable in the arm body (ComponentTagCompiler), but mir does not register
the assignment and emits UndefinedVariable. Ignored pending fix — see ROADMAP §1.4.
===ignore===
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
