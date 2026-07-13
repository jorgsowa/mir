===description===
Sibling of circular_trait_composition: a non-circular trait-composes-trait
chain must stay silent.
===file===
<?php
trait TraitA {}
trait TraitB {
    use TraitA;
}
class C {
    use TraitB;
}
===expect===
