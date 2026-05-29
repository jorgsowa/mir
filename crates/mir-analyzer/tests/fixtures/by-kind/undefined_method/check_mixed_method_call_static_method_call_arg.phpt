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
MixedMethodCall@6:5-6:22: Method bar() called on mixed type
UndefinedMethod@6:13-6:21: Method B::bat() does not exist
