===description===
No warning when an interface declares __toString and a concrete implementation is passed to a string param
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php
interface Printable {
    public function __toString();
}

class Report implements Printable {
    public function __toString() { return 'report'; }
}

function log(string $message): void {}

log(new Report());
===expect===
