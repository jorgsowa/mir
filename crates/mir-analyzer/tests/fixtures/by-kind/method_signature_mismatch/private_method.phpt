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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Method C2::f() has #[Override] but parent method C::f() is private
}
