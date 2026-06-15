===description===
ParentNotFound fires when parent:: is used in a class with no parent.
===file===
<?php
class Orphan {
    public function build(): void {
        parent::build();
    }
}
===expect===
ParentNotFound@4:8-4:14: Cannot use parent:: when current class has no parent
