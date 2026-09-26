===description===
The class in a `catch` clause resolves to the exception class.
===cursor===
symbol
===file===
<?php
try {
    echo 1;
} catch (Runtime<CURSOR>Exception $e) {
}
===expect===
kind: class RuntimeException
type: class-string
