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
ParentNotFound@4:9-4:15: Cannot use parent:: when current class has no parent
