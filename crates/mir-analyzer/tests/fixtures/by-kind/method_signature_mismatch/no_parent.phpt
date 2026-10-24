===description===
No parent
===file===
<?php
class C {
    #[Override]
    public function f(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Method C::f() has #[Override] but no parent method exists to override
}
