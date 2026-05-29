===description===
unknown return type hint
===file===
<?php
function f(): UnknownClass {
    return null;
}
===expect===
UndefinedClass@2:15-2:27: Class UnknownClass does not exist
