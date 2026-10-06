===description===
Intersection bounds check all constraints with inheritance awareness
===file:test.php===
<?php
interface Readable {
    public function read(): string;
}

interface Writable {
    public function write(string $_data): void;
}

class Base {}

class Both extends Base implements Readable, Writable {
    public function read(): string { return ''; }
    public function write(string $_data): void {}
}

class OnlyReadable extends Base implements Readable {
    public function read(): string { return ''; }
}

/**
 * @template T of Base&Readable&Writable
 * @param T $_stream
 */
function processStream($_stream): void {}
//                     ^^^^^^^^ UnusedParam: Parameter $_stream is never used

$readable = new OnlyReadable();
processStream($readable);
//<^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'OnlyReadable' does not satisfy bound 'Base&Readable&Writable'
