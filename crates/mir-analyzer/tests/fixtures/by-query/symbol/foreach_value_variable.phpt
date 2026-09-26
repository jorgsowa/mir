===description===
A foreach value variable resolves to the iterable's element type.
===cursor===
symbol
===file===
<?php
/** @param list<string> $names */
function f(array $names): void {
    foreach ($names as $name) {
        echo $na<CURSOR>me;
    }
}
===expect===
kind: variable $name
type: string
