===description===
Check mixed method call static method call arg
===file===
<?php
class B {}
/** @param mixed $a */
function foo($a) : void {
    /** @suppress MixedMethodCall */
    $a->bar(B::bat());
}
===expect===
UndefinedMethod@6:12-6:20: Method B::bat() does not exist
