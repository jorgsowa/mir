===description===
No parent
===file===
<?php
class Foo {
    public function barBar(): void {
        parent::barBar();
    }
}
===expect===
ParentNotFound@4:8-4:14: Cannot use parent:: when current class has no parent
