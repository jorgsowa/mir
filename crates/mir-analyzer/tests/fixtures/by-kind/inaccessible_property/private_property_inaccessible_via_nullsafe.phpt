===description===
InaccessibleProperty fires through the nullsafe operator (?->) exactly like a plain property fetch.
===file===
<?php
class Vault
{
    private string $secret = 'classified';
}

function reveal(?Vault $v): ?string
{
    return $v?->secret;
}
===expect===
InaccessibleProperty@9:16-9:22: Cannot access property Vault::$secret
