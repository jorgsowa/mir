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
DeprecatedTrait
===ignore===
TODO
