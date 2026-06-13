===description===
Regression (laravel/framework): stdClass permits arbitrary dynamic properties
(json_decode results, DB rows, NestedRules::compile output), so any ->prop access
is valid. mir no longer treats stdClass as sealed for property-fetch and so no
longer emits UndefinedProperty.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement,MixedAssignment,MixedPropertyFetch
===file===
<?php
function rulesOf(\stdClass $compiled): mixed {
    return $compiled->rules;
}
===expect===
