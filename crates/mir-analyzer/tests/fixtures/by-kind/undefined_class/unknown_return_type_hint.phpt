===description===
unknown return type hint
===file===
<?php
function f(): UnknownClass {
//            ^^^^^^^^^^^^ UndefinedClass: Class UnknownClass does not exist
    return null;
}
===expect===
