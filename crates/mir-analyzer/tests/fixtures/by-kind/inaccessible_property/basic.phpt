===description===
InaccessibleProperty fires when accessing a private property from outside the class.
===file===
<?php
class Vault
{
    private string $secret = 'classified';
}

echo (new Vault())->secret;
===expect===
InaccessibleProperty@7:20-7:26: Cannot access property Vault::$secret
