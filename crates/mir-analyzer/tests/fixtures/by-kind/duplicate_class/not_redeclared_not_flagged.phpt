===description===
DuplicateClass does NOT fire when a class is declared only once.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    public string $bar = '';
}

$obj = new Foo();
===expect===
