===description===
unknown return type hint
===file===
<?php
function f(): UnknownClass {
    return null;
}
===expect===
UndefinedClass@2:14-2:26: Class UnknownClass does not exist
