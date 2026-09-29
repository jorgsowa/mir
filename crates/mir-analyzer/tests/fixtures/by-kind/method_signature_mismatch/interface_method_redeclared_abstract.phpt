===description===
Interface methods are implicitly abstract; an abstract class may re-declare them `abstract`.
===config===
php_version=8.4
===file===
<?php
interface Writer {
    public function write(string $s): void;
}
abstract class BaseWriter implements Writer {
    // — interface methods are implicitly abstract)
    abstract public function write(string $s): void;
}
===expect===
