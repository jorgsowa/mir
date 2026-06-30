===description===
MixedArrayAccess still fires inside an is_array() guard because is_array() does not narrow mixed away — narrow_to_array() preserves TMixed.
===file===
<?php
function foo(mixed $a): void {
    if (is_array($a)) {
        echo $a[0];
    }
}
===expect===
MixedArrayAccess@4:13-4:18: Array access on mixed type
