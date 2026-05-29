===description===
Is class no trait
===file===
<?php
class B {}

class A {
    use B;
}
===expect===
InvalidTraitUse@5:8-5:9: Trait B used incorrectly: B is a class, not a trait
