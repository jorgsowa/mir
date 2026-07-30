===description===
InaccessibleProperty does NOT fire when a private property is accessed from within its declaring class.
===file===
<?php
class Vault
{
    private string $secret = 'classified';

    public function reveal(): string
    {
        return $this->secret;
    }
}
===expect===
