===description===
Deprecated trait
===file===
<?php
/** @deprecated */
trait T {}

class C {
    use T;
}

===expect===
DeprecatedTrait@5:0-5:9: Trait T is deprecated
