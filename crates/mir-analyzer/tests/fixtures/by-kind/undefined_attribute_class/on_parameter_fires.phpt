===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a function parameter.
===file===
<?php
function foo(#[Inject] string $svc): string {
    return $svc;
}
===expect===
UndefinedAttributeClass@2:15-2:21: Attribute class Inject does not exist
