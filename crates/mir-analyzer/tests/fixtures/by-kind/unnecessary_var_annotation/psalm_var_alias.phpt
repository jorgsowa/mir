===description===
@psalm-var must be recognized as an alias of @var, the same way @psalm-
template/@phpstan-template already alias @template.
===file===
<?php
function foo(): string {
    return "hello";
}

/** @psalm-var string */
$a = foo();
//<^^^^^^^^^^^ UnnecessaryVarAnnotation: @var annotation for $a is unnecessary

echo $a;
