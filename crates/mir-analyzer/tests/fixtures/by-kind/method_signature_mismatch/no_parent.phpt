===description===
No parent
===file===
<?php
class C {
    #[Override]
//  ^^^^^^^^^^^ InvalidOverride: Method C::f() has #[Override] but no parent method exists to override
    public function f(): void {}
}
