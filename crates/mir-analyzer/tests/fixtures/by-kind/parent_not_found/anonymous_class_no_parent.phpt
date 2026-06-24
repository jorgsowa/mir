===description===
ParentNotFound fires when parent:: is used inside an anonymous class that has
no extends clause.
===config===
suppress=UnusedVariable
===file===
<?php

$obj = new class {
    public function build(): void {
        parent::build();
    }
};
===expect===
ParentNotFound@5:8-5:14: Cannot use parent:: when current class has no parent
