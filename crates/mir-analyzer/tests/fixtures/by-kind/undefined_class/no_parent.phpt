===description===
No parent
===file===
<?php
class Foo {
    public function barBar(): void {
        parent::barBar();
//      ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
    }
}
