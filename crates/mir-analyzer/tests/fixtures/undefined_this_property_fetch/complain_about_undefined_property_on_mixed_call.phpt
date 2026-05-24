===description===
complainAboutUndefinedPropertyOnMixedCall
===file===
<?php
class C {
    /** @param mixed $a */
    public function foo($a) : void {
        /** @psalm-suppress MixedMethodCall */
        $a->bar($this->d);
    }
}
===expect===
UndefinedThisPropertyFetch
===ignore===
TODO
