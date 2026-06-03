===description===
Interface with no parent
===file===
<?php
interface I {
    #[Override]
    public function f(): void;
}

===expect===
InvalidOverride@3:4-3:15: Method I::f() has #[Override] but no parent method exists to override
