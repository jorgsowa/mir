===description===
Private method
===file===
<?php
class C {
    private function f(): void {}
}

class C2 extends C {
    #[Override]
    private function f(): void {}
}

===expect===
InvalidOverride
===ignore===
TODO
