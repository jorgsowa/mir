===description===
InaccessibleProperty fires when a subclass tries to access a private property declared on its parent.
===file===
<?php
class Base
{
    private string $secret = 'hidden';
}

class Child extends Base
{
    public function getSecret(): string
    {
        return $this->secret;
//                    ^^^^^^ InaccessibleProperty: Cannot access property Base::$secret
    }
}
===expect===
