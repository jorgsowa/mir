===description===
Undefined trait
===file===
<?php
class B {
    use A;
//      ^ UndefinedTrait: Trait A does not exist
}
===expect===
