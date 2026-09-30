===description===
InaccessibleProperty fires when a private property is accessed through an object-typed parameter from outside the declaring class.
===file===
<?php
class Vault
{
    private string $secret = 'classified';
}

function reveal(Vault $v): string
{
    return $v->secret;
//             ^^^^^^ InaccessibleProperty: Cannot access property Vault::$secret
}
===expect===
