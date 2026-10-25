===description===
No parent
===file===
<?php
class C {
    #[Override]
    public function f(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: C::f() has #[\Override] attribute, but no matching parent method exists
}
