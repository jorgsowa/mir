===description===
Is class no trait
===file===
<?php
class B {}

class A {
    use B;
//      ^ InvalidTraitUse: Trait B used incorrectly: B is a class, not a trait
}
===expect===
