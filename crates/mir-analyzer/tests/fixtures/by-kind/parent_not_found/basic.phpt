===description===
ParentNotFound fires when parent:: is used in a class with no parent.
===file===
<?php
class Orphan {
    public function build(): void {
        parent::build();
//      ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
    }
}
===expect===
