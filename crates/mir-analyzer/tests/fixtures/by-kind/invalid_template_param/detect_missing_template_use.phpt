===description===
Detect missing template use
===ignore===
TODO
===file===
<?php
/** @template T */
trait A {}
final class B {
    use A;
}

===expect===
