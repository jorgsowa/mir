===description===
interfaceWithNoParent
===file===
<?php
interface I {
    #[Override]
    public function f(): void;
}

===expect===
InvalidOverride
===ignore===
TODO
