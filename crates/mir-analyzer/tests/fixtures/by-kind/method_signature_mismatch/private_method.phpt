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
InvalidOverride@7:4-7:15: Method C2::f() has #[Override] but parent method C::f() is private
