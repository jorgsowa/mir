===description===
Circular trait composition (trait A { use B; } trait B { use A; }) was
invisible to cycle detection, unlike class/interface inheritance cycles.
===file===
<?php
trait TraitA {
    use TraitB;
}
trait TraitB {
    use TraitA;
}
===expect===
InvalidTraitUse@5:0-5:14: Trait TraitB used incorrectly: TraitB has a circular trait composition chain
