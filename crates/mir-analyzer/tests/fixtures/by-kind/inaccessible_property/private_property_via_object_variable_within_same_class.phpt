===description===
InaccessibleProperty does NOT fire when a private property is accessed on another instance of the same declaring class.
===file===
<?php
class Vault
{
    private string $secret = 'classified';

    public function sameSecretAs(Vault $other): bool
    {
        return $this->secret === $other->secret;
    }
}
===expect===
