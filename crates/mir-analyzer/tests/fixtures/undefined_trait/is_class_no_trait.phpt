===description===
isClassNoTrait
===file===
<?php
class B {}

class A {
    use B;
}
===expect===
InvalidTraitUse@5:8: Trait B used incorrectly: B is a class, not a trait
