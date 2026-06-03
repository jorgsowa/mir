===description===
No parent
===file===
<?php
class C {
    #[Override]
    public function f(): void {}
}

===expect===
InvalidOverride@3:4-3:15: Method C::f() has #[Override] but no parent method exists to override
