===description===
Laravel FP (laravel/framework): `preg_split()` returns array|false; an access
guarded by `isset($matches[1])` is safe, but mir does not narrow false/null out of
an array-access target under isset() and emits PossiblyInvalidArrayAccess
(Console\Parser). Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement,MixedAssignment
===file===
<?php
function firstWord(string $token): string {
    $matches = preg_split('/\s+/', $token);
    if (isset($matches[1])) {
        return $matches[1];
    }
    return '';
}
===expect===
