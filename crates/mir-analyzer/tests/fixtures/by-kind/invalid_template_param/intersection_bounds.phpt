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

$readable = new OnlyReadable();
processStream($readable);
===expect===
test.php: UnusedParam@14:27-14:40: Parameter $_data is never used
test.php: UnusedParam@25:24-25:32: Parameter $_stream is never used
test.php: InvalidTemplateParam@28:1-28:25: Template type 'T' inferred as 'OnlyReadable' does not satisfy bound 'Base&Readable&Writable'
