===description===
Private method
===file===
<?php
class C {
    private function f(): void {}
}

class C2 extends C {
    #[Override]
//  ^^^^^^^^^^^ InvalidOverride: Method C2::f() has #[Override] but parent method C::f() is private
    private function f(): void {}
}

===expect===
