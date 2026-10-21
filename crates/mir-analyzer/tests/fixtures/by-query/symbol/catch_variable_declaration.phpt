===description===
The variable bound by a catch clause resolves at its declaration.
===cursor===
symbol
===file===
<?php
class Boom extends Exception {}

function f(): void {
    try {
        throw new Boom();
    } catch (Boom $e<CURSOR>rr) {
        echo $err->getMessage();
    }
}
===expect===
kind: variable $err
type: Boom
