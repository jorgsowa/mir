===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a function parameter.
===file===
<?php
function foo(#[Inject] string $svc): string {
//             ^^^^^^ UndefinedAttributeClass: Attribute class Inject does not exist
    return $svc;
}
===expect===
