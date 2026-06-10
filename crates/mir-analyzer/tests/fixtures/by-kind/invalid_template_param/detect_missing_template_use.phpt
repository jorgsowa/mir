===description===
Detect missing template use
===file===
<?php
/** @template T */
trait A {}
final class B {
    use A;
}

===expect===
