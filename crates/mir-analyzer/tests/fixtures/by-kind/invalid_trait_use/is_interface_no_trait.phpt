===description===
Is interface no trait
===file===
<?php
interface B {}

class A {
    use B;
//      ^ InvalidTraitUse: Trait B used incorrectly: B is an interface, not a trait
}
