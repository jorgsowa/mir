===description===
Child redeclares parent property with same native type — no error
===file===
<?php
class A {
    public int $count = 0;
    public string $name = '';
    public ?bool $flag = null;
}

class B extends A {
    public int $count = 1;
    public string $name = 'default';
    public ?bool $flag = false;
}
===expect===
