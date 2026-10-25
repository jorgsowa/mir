===description===
Interface with no parent
===file===
<?php
interface I {
    #[Override]
    public function f(): void;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: I::f() has #[\Override] attribute, but no matching parent method exists
}
