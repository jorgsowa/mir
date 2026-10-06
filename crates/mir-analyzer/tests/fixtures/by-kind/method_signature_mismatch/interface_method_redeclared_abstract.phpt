===description===
Interface methods are implicitly abstract; an abstract class may re-declare them `abstract`.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
interface Writer {
    public function write(string $s): void;
}
abstract class BaseWriter implements Writer {
    // — interface methods are implicitly abstract)
    abstract public function write(string $s): void;
}
