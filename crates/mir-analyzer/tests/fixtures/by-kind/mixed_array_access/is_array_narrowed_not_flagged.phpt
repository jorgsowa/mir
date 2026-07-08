===description===
MixedArrayAccess does not fire inside an is_array() guard — is_array() narrows mixed to a concrete array type.
===file===
<?php
function foo(mixed $a): void {
    if (is_array($a)) {
        echo $a[0];
    }
}
===expect===
