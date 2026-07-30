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
}
===expect===
InaccessibleProperty@9:15-9:21: Cannot access property Vault::$secret
