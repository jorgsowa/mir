===description===
DeprecatedTrait does NOT fire for traits without the @deprecated annotation.
===file===
<?php
trait NotDeprecated {
    public function helper(): string { return "ok"; }
}

class UsesIt {
    use NotDeprecated;
}

===expect===
