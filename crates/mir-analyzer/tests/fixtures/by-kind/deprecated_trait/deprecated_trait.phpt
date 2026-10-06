===description===
Deprecated trait
===file===
<?php
/** @deprecated */
trait T {}

class C {
//<^^^^^^^^^ DeprecatedTrait: Trait T is deprecated
    use T;
}
