===description===
Anonymous class that implements an interface (but does not extend a class) still
emits ParentNotFound for parent:: calls — implements does not establish a parent.
===config===
suppress=UnusedVariable,MixedReturnStatement
===file===
<?php

interface Greeter {
    public function greet(): string;
}

$obj = new class implements Greeter {
    public function greet(): string {
        return parent::greet(); // No parent class — should error
    }
};
===expect===
ParentNotFound@9:15-9:21: Cannot use parent:: when current class has no parent
