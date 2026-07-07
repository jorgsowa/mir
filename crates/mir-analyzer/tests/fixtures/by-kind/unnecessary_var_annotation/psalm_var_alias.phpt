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

echo $a;
===expect===
UnnecessaryVarAnnotation@7:0-7:11: @var annotation for $a is unnecessary
