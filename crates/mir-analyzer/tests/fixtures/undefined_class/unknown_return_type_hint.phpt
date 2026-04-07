===source===
<?php
function f(): UnknownClass {
    return null;
}
===expect===
UndefinedClass: UnknownClass
