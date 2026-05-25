===description===
Is interface no trait
===file===
<?php
interface B {}

class A {
    use B;
}
===expect===
InvalidTraitUse@5:8: Trait B used incorrectly: B is an interface, not a trait
