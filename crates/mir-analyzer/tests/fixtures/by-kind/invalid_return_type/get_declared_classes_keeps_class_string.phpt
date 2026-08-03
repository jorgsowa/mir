===description===
M18: get_declared_classes() was typed bare `array` (elements plain
string), so returning it from a method declared @return array<class-
string> flagged InvalidReturnType. Fixed by typing it list<class-string>,
matching what it actually yields.
===file===
<?php
/** @return array<class-string> */
function declaredClasses(): array {
    return get_declared_classes();
}
===expect===
