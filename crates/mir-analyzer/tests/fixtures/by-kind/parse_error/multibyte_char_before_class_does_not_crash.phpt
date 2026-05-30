===description===
multibyte char before class does not crash
===file===
<?php
// This comment contains a multibyte arrow → symbol before the class declaration.
// It must not cause a panic when looking for the preceding docblock.
class Foo {}
===expect===
