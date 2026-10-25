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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: C2::f() has #[\Override] attribute, but no matching parent method exists
}
