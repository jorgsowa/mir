===description===
Interface with no parent
===file===
<?php
interface I {
    #[Override]
//  ^^^^^^^^^^^ InvalidOverride: Method I::f() has #[Override] but no parent method exists to override
    public function f(): void;
}
