===description===
unknown return type hint
===file===
<?php
function f(): UnknownClass {
    return null;
}
===expect===
UndefinedClass: Class UnknownClass does not exist
===ignore===
TODO
