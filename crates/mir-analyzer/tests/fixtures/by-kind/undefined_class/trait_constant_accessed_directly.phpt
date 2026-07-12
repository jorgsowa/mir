===description===
A trait's constant fetched directly through the trait (rather than a class
that uses it) is a PHP fatal error regardless of whether the constant
exists, since a trait is never a valid constant-access target.
===file===
<?php
trait HasFoo {
    const FOO = 1;
}

echo HasFoo::FOO;
===expect===
TraitConstantAccessedDirectly@6:5-6:16: Cannot access trait constant HasFoo::FOO directly
