===description===
A trait constant accessed through a class that uses the trait (not the
trait itself) is valid PHP 8.2+ and must not report
TraitConstantAccessedDirectly.
===file===
<?php
trait HasFoo {
    const FOO = 1;
}

class Bar {
    use HasFoo;
}

echo Bar::FOO;
===expect===
