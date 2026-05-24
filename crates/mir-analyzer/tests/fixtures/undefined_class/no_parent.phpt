===description===
noParent
===file===
<?php
class Foo {
    public function barBar(): void {
        parent::barBar();
    }
}
===expect===
ParentNotFound
===ignore===
TODO
