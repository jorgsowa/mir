===description===
Laravel FP (laravel/framework): stdClass permits arbitrary dynamic properties
(json_decode results, DB rows, NestedRules::compile output), so any ->prop access
is valid, but mir treats stdClass as sealed and emits UndefinedProperty. Ignored
pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement,MixedAssignment,MixedPropertyFetch
===file===
<?php
function rulesOf(\stdClass $compiled): mixed {
    return $compiled->rules;
}
===expect===
